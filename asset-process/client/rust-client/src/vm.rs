//! Client-script core recovered from bj.cl and the injected eo interpreter.
//! Objects and arrays retain reference identity across calls, as Java does.
use crate::{
    script::{Operand, Script},
    Error, Result,
};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug)]
pub enum Object {
    Null,
    String(String),
    Array(Rc<RefCell<Array>>),
}
#[derive(Clone, Debug, PartialEq)]
pub enum Array {
    Managed(Box<crate::script_array_storage::Storage>),
    Longs(Vec<i64>),
    Ints(Vec<i32>),
    Strings(Vec<Option<String>>),
}
#[derive(Default, Debug)]
pub struct Stacks {
    int_slots: Rc<RefCell<Vec<i32>>>,
    ints: Vec<i32>,
    objects: Vec<Object>,
    longs: Vec<i64>,
}
impl Stacks {
    pub fn int_slot0(&self) -> i32 {
        self.int_slots.borrow().first().copied().unwrap_or(0)
    }
    pub fn seed_int_slot0(&mut self, value: i32) {
        let mut slots = self.int_slots.borrow_mut();
        if slots.is_empty() { slots.push(value); } else { slots[0] = value; }
    }
    pub fn ints(&self) -> &[i32] {
        &self.ints
    }
    pub fn objects(&self) -> &[Object] {
        &self.objects
    }
    pub fn longs(&self) -> &[i64] {
        &self.longs
    }
    pub fn push_int(&mut self, value: i32) -> Result<()> {
        let index = self.ints.len();
        if index >= 1000 { return Err(Error::Invalid("script stack overflow")); }
        let mut slots = self.int_slots.borrow_mut();
        if index >= slots.len() { slots.resize(index + 1, 0); }
        slots[index] = value;
        self.ints.push(value);
        Ok(())
    }
    /// Java array stack increments its pointer before evaluating an assignment RHS.
    /// A failed parse leaves the previously stored value at this physical slot.
    pub fn reserve_int(&mut self) -> Result<()> {
        let value = self.int_slots.borrow().get(self.ints.len()).copied().unwrap_or(0);
        self.push_int(value)
    }
    pub fn push_object(&mut self, value: Object) -> Result<()> {
        push(&mut self.objects, value)
    }
    pub fn push_long(&mut self, value: i64) -> Result<()> {
        push(&mut self.longs, value)
    }
    pub fn pop_int(&mut self) -> Result<i32> {
        pop(&mut self.ints)
    }
    pub fn pop_object(&mut self) -> Result<Object> {
        pop(&mut self.objects)
    }
    pub fn pop_long(&mut self) -> Result<i64> {
        pop(&mut self.longs)
    }
}
fn push<T>(stack: &mut Vec<T>, value: T) -> Result<()> {
    if stack.len() >= 1000 {
        return Err(Error::Invalid("script stack overflow"));
    }
    stack.push(value);
    Ok(())
}
fn pop<T>(stack: &mut Vec<T>) -> Result<T> {
    stack.pop().ok_or(Error::Invalid("script stack underflow"))
}
fn take<T>(stack: &mut Vec<T>, count: usize) -> Result<Vec<T>> {
    let start = stack
        .len()
        .checked_sub(count)
        .ok_or(Error::Invalid("script argument underflow"))?;
    Ok(stack.split_off(start))
}
fn index(value: i32) -> Result<usize> {
    usize::try_from(value).map_err(|_| Error::Invalid("negative script index"))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variable {
    Player,
    Bit,
    ClientInt,
    ClientString,
    ClientLong,
    ClanSetting,
    Clan,
}
#[derive(Clone, Debug)]
pub enum Value {
    Int(i32),
    Long(i64),
    Object(Object),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
    Continue,
    Halt,
    RequestClose,
}
pub trait Host {
    fn begin_script(&mut self) {}
    fn request_close(&mut self) {}
    fn take_close_request(&mut self) -> bool { true }
    fn script(&mut self, id: i32) -> Result<Rc<Script>>;
    fn read(&mut self, kind: Variable, id: i32) -> Result<Value>;
    fn write(&mut self, kind: Variable, id: i32, value: Value) -> Result<()>;
    fn extended(&mut self, opcode: u16, dot_widget: bool, stacks: &mut Stacks) -> Result<Control>;
    fn finish_script(&mut self) -> Result<()> { Ok(()) }
    fn close_interfaces(&mut self) -> Result<()> {
        Err(Error::Unsupported("script close interfaces"))
    }
}
struct Frame {
    script: Rc<Script>,
    pc: usize,
    ints: Vec<i32>,
    objects: Vec<Object>,
    longs: Vec<i64>,
}
impl Frame {
    fn new(
        script: Rc<Script>,
        ints: Vec<i32>,
        objects: Vec<Object>,
        longs: Vec<i64>,
    ) -> Result<Self> {
        if ints.len() > script.local_ints as usize
            || objects.len() > script.local_strings as usize
            || longs.len() > script.local_longs as usize
        {
            return Err(Error::Invalid("script arguments exceed locals"));
        }
        let mut frame = Self {
            ints,
            objects,
            longs,
            script,
            pc: 0,
        };
        frame.ints.resize(frame.script.local_ints as usize, 0);
        frame
            .objects
            .resize(frame.script.local_strings as usize, Object::Null);
        frame.longs.resize(frame.script.local_longs as usize, 0);
        Ok(frame)
    }
    fn branch(&mut self, offset: i32) -> Result<()> {
        self.pc = self
            .pc
            .checked_add_signed(offset as isize)
            .filter(|&p| p < self.script.instructions.len())
            .ok_or(Error::Invalid("script branch target"))?;
        Ok(())
    }
    fn array(&self, local: usize) -> Result<Rc<RefCell<Array>>> {
        match self.objects.get(local) {
            Some(Object::Array(a)) => Ok(a.clone()),
            _ => Err(Error::Invalid("script local is not an array")),
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    Yielded,
    Finished,
}
pub struct Vm {
    current: Frame,
    frames: Vec<Frame>,
    pub stacks: Stacks,
    executed: u64,
    limit: u64,
    finished: bool,
    failed: bool,
    close_requested: bool,
    completion_called: bool,
    started: bool,
}
impl Vm {
    /// Original bj.ab storage survives top-level script executions; only depth resets.
    pub fn with_int_slots(mut self, slots: Rc<RefCell<Vec<i32>>>) -> Self {
        self.stacks.int_slots = slots;
        self
    }

    pub fn with_int_slot0(mut self, value: i32) -> Self {
        self.stacks.seed_int_slot0(value);
        self
    }
    /// Event arguments have already had their sentinel values substituted.
    pub fn new(
        script: Rc<Script>,
        ints: Vec<i32>,
        objects: Vec<Object>,
        longs: Vec<i64>,
        instruction_limit: u64,
    ) -> Result<Self> {
        Ok(Self {
            current: Frame::new(script, ints, objects, longs)?,
            frames: Vec::new(),
            stacks: Stacks::default(),
            executed: 0,
            limit: instruction_limit,
            finished: false,
            failed: false,
            close_requested: false,
            completion_called: false,
            started: false,
        })
    }
    pub fn executed(&self) -> u64 {
        self.executed
    }
    pub fn run(&mut self, host: &mut impl Host, budget: usize) -> Result<Status> {
        if self.failed {
            return Err(Error::Invalid("failed script VM"));
        }
        if !std::mem::replace(&mut self.started, true) { host.begin_script(); }
        let result = self.run_inner(host, budget);
        if result.is_err() {
            self.failed = true;
        }
        // bj.cl's finally path runs once, including script errors and instruction limits.
        // A scheduler yield must not commit the close request.
        if !matches!(result, Ok(Status::Yielded)) && !std::mem::replace(&mut self.completion_called, true) {
            if let Err(error) = host.finish_script() {
                self.failed = true;
                return Err(error);
            }
        }
        if !matches!(result, Ok(Status::Yielded)) && std::mem::take(&mut self.close_requested) && host.take_close_request() {
            if let Err(error) = host.close_interfaces() {
                self.failed = true;
                if result.is_ok() { return Err(error); }
            }
        }
        result
    }
    fn run_inner(&mut self, host: &mut impl Host, budget: usize) -> Result<Status> {
        for _ in 0..budget {
            if self.finished {
                return Ok(Status::Finished);
            }
            if self.executed >= self.limit {
                return Err(Error::Invalid("script instruction limit"));
            }
            let instruction = self
                .current
                .script
                .instructions
                .get(self.current.pc)
                .ok_or(Error::Invalid("script program counter"))?
                .clone();
            self.current.pc += 1;
            self.executed += 1;
            let op = instruction.opcode;
            if let Operand::String(value) = instruction.operand {
                if op != 3 {
                    return Err(Error::Invalid("script string operand"));
                }
                self.stacks.push_object(Object::String(value))?;
                continue;
            }
            if let Operand::Long(value) = instruction.operand {
                if op != 61 {
                    return Err(Error::Invalid("script long operand"));
                }
                self.stacks.push_long(value)?;
                continue;
            }
            let Operand::Int(arg) = instruction.operand else {
                unreachable!()
            };
            match op {
                0 => self.stacks.push_int(arg)?,
                1 | 25 | 42 | 49 | 51 | 74 | 76 => {
                    let kind = match op {
                        1 => Variable::Player,
                        25 => Variable::Bit,
                        42 => Variable::ClientInt,
                        49 => Variable::ClientString,
                        51 => Variable::ClientLong,
                        74 => Variable::ClanSetting,
                        _ => Variable::Clan,
                    };
                    match (op, host.read(kind, arg)?) {
                        (49, Value::Object(v)) => self.stacks.push_object(v)?,
                        (51, Value::Long(v)) => self.stacks.push_long(v)?,
                        (1 | 25 | 42 | 74 | 76, Value::Int(v)) => self.stacks.push_int(v)?,
                        _ => return Err(Error::Invalid("script variable type")),
                    }
                }
                2 | 27 | 43 | 50 | 52 => {
                    let kind = match op {
                        2 => Variable::Player,
                        27 => Variable::Bit,
                        43 => Variable::ClientInt,
                        50 => Variable::ClientString,
                        _ => Variable::ClientLong,
                    };
                    let value = match op {
                        50 => Value::Object(self.stacks.pop_object()?),
                        52 => Value::Long(self.stacks.pop_long()?),
                        _ => Value::Int(self.stacks.pop_int()?),
                    };
                    host.write(kind, arg, value)?;
                }
                6 => self.current.branch(arg)?,
                7 | 8 | 9 | 10 | 31 | 32 => {
                    let b = self.stacks.pop_int()?;
                    let a = self.stacks.pop_int()?;
                    if match op {
                        7 => a != b,
                        8 => a == b,
                        9 => a < b,
                        10 => a > b,
                        31 => a <= b,
                        _ => a >= b,
                    } {
                        self.current.branch(arg)?;
                    }
                }
                68..=73 => {
                    let b = self.stacks.pop_long()?;
                    let a = self.stacks.pop_long()?;
                    if match op {
                        68 => a != b,
                        69 => a == b,
                        70 => a < b,
                        71 => a > b,
                        72 => a <= b,
                        _ => a >= b,
                    } {
                        self.current.branch(arg)?;
                    }
                }
                21 => {
                    if let Some(frame) = self.frames.pop() {
                        self.current = frame;
                    } else {
                        self.finished = true;
                    }
                }
                33 => self.stacks.push_int(
                    *self
                        .current
                        .ints
                        .get(index(arg)?)
                        .ok_or(Error::Invalid("integer local"))?,
                )?,
                34 => {
                    *self
                        .current
                        .ints
                        .get_mut(index(arg)?)
                        .ok_or(Error::Invalid("integer local"))? = self.stacks.pop_int()?
                }
                35 => self.stacks.push_object(
                    self.current
                        .objects
                        .get(index(arg)?)
                        .ok_or(Error::Invalid("object local"))?
                        .clone(),
                )?,
                36 => {
                    *self
                        .current
                        .objects
                        .get_mut(index(arg)?)
                        .ok_or(Error::Invalid("object local"))? = self.stacks.pop_object()?
                }
                66 => self.stacks.push_long(
                    *self
                        .current
                        .longs
                        .get(index(arg)?)
                        .ok_or(Error::Invalid("long local"))?,
                )?,
                67 => {
                    *self
                        .current
                        .longs
                        .get_mut(index(arg)?)
                        .ok_or(Error::Invalid("long local"))? = self.stacks.pop_long()?
                }
                37 => {
                    let values = take(&mut self.stacks.objects, index(arg)?)?;
                    let mut joined = String::new();
                    for value in values {
                        match value {
                            Object::Null => joined.push_str("null"),
                            Object::String(s) => joined.push_str(&s),
                            _ => return Err(Error::Invalid("join requires strings")),
                        }
                    }
                    self.stacks.push_object(Object::String(joined))?;
                }
                38 => {
                    self.stacks.pop_int()?;
                }
                39 => {
                    self.stacks.pop_object()?;
                }
                62 => {
                    self.stacks.pop_long()?;
                }
                63 => self.stacks.push_object(Object::Null)?,
                40 => {
                    if self.frames.len() >= 50 {
                        return Err(Error::Invalid("script call depth"));
                    }
                    let script = host.script(arg)?;
                    let ints = take(&mut self.stacks.ints, script.int_arguments as usize)?;
                    let objects = take(&mut self.stacks.objects, script.string_arguments as usize)?;
                    let longs = take(&mut self.stacks.longs, script.long_arguments as usize)?;
                    let next = Frame::new(script, ints, objects, longs)?;
                    self.frames.push(std::mem::replace(&mut self.current, next));
                }
                44 => {
                    let local = index(arg >> 16)?;
                    let kind = arg & 65535;
                    let size = self.stacks.pop_int()?;
                    if !(0..=5000).contains(&size) {
                        return Err(Error::Invalid("script array size"));
                    }
                    let array = Array::create(kind, size, size)?;
                    *self
                        .current
                        .objects
                        .get_mut(local)
                        .ok_or(Error::Invalid("array local"))? =
                        Object::Array(Rc::new(RefCell::new(array)));
                }
                45 => {
                    let array = self.current.array(index(arg)?)?;
                    let at = index(self.stacks.pop_int()?)?;
                    match array.borrow().data() {
                        Array::Managed(_) => unreachable!(),
                        Array::Longs(_) => {
                            return Err(Error::Invalid("legacy array read on long array"))
                        }
                        Array::Ints(v) => self
                            .stacks
                            .push_int(*v.get(at).ok_or(Error::Invalid("array index"))?)?,
                        Array::Strings(v) => self.stacks.push_object(
                            v.get(at)
                                .ok_or(Error::Invalid("array index"))?
                                .clone()
                                .map_or(Object::Null, Object::String),
                        )?,
                    };
                }
                46 => {
                    let array = self.current.array(index(arg)?)?;
                    array.borrow().writable()?;
                    match array.borrow_mut().data_mut() {
                        Array::Managed(_) => unreachable!(),
                        Array::Longs(_) => {
                            return Err(Error::Invalid("legacy array write on long array"))
                        }
                        Array::Ints(v) => {
                            let value = self.stacks.pop_int()?;
                            let at = index(self.stacks.pop_int()?)?;
                            *v.get_mut(at).ok_or(Error::Invalid("array index"))? = value;
                        }
                        Array::Strings(v) => {
                            let at = index(self.stacks.pop_int()?)?;
                            let value = match self.stacks.pop_object()? {
                                Object::Null => None,
                                Object::String(s) => Some(s),
                                _ => return Err(Error::Invalid("string array value")),
                            };
                            *v.get_mut(at).ok_or(Error::Invalid("array index"))? = value;
                        }
                    };
                }
                60 => {
                    let key = self.stacks.pop_int()?;
                    let jump = self
                        .current
                        .script
                        .switches
                        .get(index(arg)?)
                        .ok_or(Error::Invalid("switch table"))?
                        .get(&key)
                        .copied();
                    if let Some(jump) = jump {
                        self.current.branch(jump)?;
                    }
                }
                100.. => {
                    match host.extended(op, arg == 1, &mut self.stacks)? {
                        Control::Continue => {},
                        Control::Halt => self.finished = true,
                        Control::RequestClose => { self.close_requested = true; host.request_close(); },
                    }
                }
                _ => return Err(Error::Unsupported("core script opcode")),
            }
        }
        Ok(if self.finished {
            Status::Finished
        } else {
            Status::Yielded
        })
    }
}
