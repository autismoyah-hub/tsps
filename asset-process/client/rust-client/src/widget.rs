//! Legacy and IF3 interface definitions from GameWidget.aa/as.
use crate::{buffer::Reader, disk_cache::DiskCache, Error, Result};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Argument {
    Int(i32),
    Text(String),
    Null,
}
#[derive(Clone, Debug)]
pub struct Widget {
    pub trailing: Vec<u8>,
    pub id: u32,
    pub if3: bool,
    pub kind: u8,
    pub button: u8,
    pub content: u16,
    pub raw_position: [i32; 2],
    pub raw_size: [i32; 2],
    pub size_alignment: [i8; 2],
    pub position_alignment: [i8; 2],
    pub parent: Option<u32>,
    pub hidden: bool,
    pub mouse_over: i32,
    pub scroll_size: [i32; 2],
    pub no_click_through: bool,
    pub no_scroll_through: bool,
    pub colors: [u32; 4],
    pub filled: bool,
    pub transparency: u8,
    pub transparency_bottom: i32,
    pub fill_mode: u8,
    pub line_width: u8,
    pub line_reversed: bool,
    pub sprites: [i32; 2],
    pub sprite_angle: u16,
    pub sprite_tiling: bool,
    pub outline: u8,
    pub sprite_shadow: u32,
    pub flip_vertical: bool,
    pub flip_horizontal: bool,
    pub models: [i32; 2],
    pub sequences: [i32; 2],
    pub model_offset: [i32; 2],
    pub model_angles: [u16; 3],
    pub model_zoom: u16,
    pub model_orthographic: bool,
    pub model_transparent: bool,
    pub model_extra: u16,
    pub model_width_scale: u16,
    pub model_height_scale: u16,
    pub font: i32,
    pub text: String,
    pub alternate_text: String,
    pub text_alignment: [u8; 2],
    pub line_height: u8,
    pub text_shadow: bool,
    pub flags: u32,
    pub data_text: String,
    pub actions: Vec<String>,
    pub operation_priority: i32,
    pub sub_actions: Vec<Option<Vec<String>>>,
    pub drag_zone: i32,
    pub drag_threshold: i32,
    pub scrollbar: bool,
    pub spell_action: String,
    pub spell_name: String,
    pub button_text: String,
    pub listeners: [Option<Vec<Argument>>; 18],
    pub triggers: [Option<Vec<i32>>; 3],
    pub comparisons: Vec<(u8, u16)>,
    pub cs1: Vec<Vec<i32>>,
}
fn nullable(r: &mut Reader<'_>) -> Result<i32> {
    let id = r.u16()?;
    Ok(if id == 65535 { -1 } else { id as i32 })
}
impl Widget {
    pub fn set_sub_operation(&mut self, operation: i32, sub: i32, text: String) -> Result<()> {
        if !(1..=10).contains(&operation) || sub < 1 {
            return Err(Error::Invalid("widget sub-operation index"));
        }
        let operation = operation as usize - 1;
        let sub = sub as usize - 1;
        if sub > 65535 {
            return Err(Error::Unsupported("oversized widget sub-operation list"));
        }
        self.sub_actions
            .resize(self.sub_actions.len().max(operation + 1), None);
        let row = self.sub_actions[operation].get_or_insert_with(Vec::new);
        row.resize(row.len().max(sub + 1), String::new());
        row[sub] = text;
        Ok(())
    }
    pub fn clear_sub_operations(&mut self, operation: i32) -> Result<()> {
        if (1..=10).contains(&operation) && !self.sub_actions.is_empty() {
            *self
                .sub_actions
                .get_mut(operation as usize - 1)
                .ok_or(Error::Invalid("widget sub-operation row"))? = None;
        }
        Ok(())
    }
    pub fn set_operation_priority(&mut self, value: i32) {
        if value == -1 {
            self.operation_priority = 4;
        } else if (1..=32).contains(&value) {
            self.operation_priority = value - 1;
        }
    }
    pub fn set_operation(&mut self, index: i32, text: String) {
        if (1..=32).contains(&index) {
            self.actions
                .resize(self.actions.len().max(index as usize), String::new());
            self.actions[index as usize - 1] = text;
        }
    }

    pub fn empty(id: u32, kind: u8) -> Self {
        Self {
            trailing: Vec::new(),
            id,
            if3: true,
            kind,
            button: 0,
            content: 0,
            raw_position: [0; 2],
            raw_size: [0; 2],
            size_alignment: [0; 2],
            position_alignment: [0; 2],
            parent: None,
            hidden: false,
            mouse_over: -1,
            scroll_size: [0; 2],
            no_click_through: false,
            no_scroll_through: false,
            colors: [0; 4],
            filled: false,
            transparency: 0,
            transparency_bottom: 0,
            fill_mode: 0,
            line_width: 1,
            line_reversed: false,
            sprites: [-1; 2],
            sprite_angle: 0,
            sprite_tiling: false,
            outline: 0,
            sprite_shadow: 0,
            flip_vertical: false,
            flip_horizontal: false,
            models: [-1; 2],
            sequences: [-1; 2],
            model_offset: [0; 2],
            model_angles: [0; 3],
            model_zoom: 100,
            model_orthographic: false,
            model_transparent: false,
            model_extra: 0,
            model_width_scale: 0,
            model_height_scale: 0,
            font: -1,
            text: String::new(),
            alternate_text: String::new(),
            text_alignment: [0; 2],
            line_height: 0,
            text_shadow: false,
            flags: 0,
            data_text: String::new(),
            actions: Vec::new(),
            operation_priority: 4,
            sub_actions: Vec::new(),
            drag_zone: 0,
            drag_threshold: 0,
            scrollbar: false,
            spell_action: String::new(),
            spell_name: String::new(),
            button_text: "Ok".into(),
            listeners: std::array::from_fn(|_| None),
            triggers: std::array::from_fn(|_| None),
            comparisons: Vec::new(),
            cs1: Vec::new(),
        }
    }

    pub fn decode(id: u32, data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let if3 = data.first() == Some(&255);
        if if3 {
            r.u8()?;
        }
        let mut w = Self::empty(id, r.u8()?);
        w.if3 = if3;
        if !if3 {
            w.button = r.u8()?;
        }
        w.content = r.u16()?;
        w.raw_position = [r.u16()? as i16 as i32, r.u16()? as i16 as i32];
        w.raw_size = [
            r.u16()? as i32,
            if if3 && w.kind == 9 {
                r.u16()? as i16 as i32
            } else {
                r.u16()? as i32
            },
        ];
        if if3 {
            w.size_alignment = [r.u8()? as i8, r.u8()? as i8];
            w.position_alignment = [r.u8()? as i8, r.u8()? as i8];
        } else {
            w.transparency = r.u8()?;
        }
        let parent = nullable(&mut r)?;
        w.parent = (parent != -1).then_some((id & 0xffff0000) | (parent as u32));
        if if3 {
            w.hidden = r.u8()? == 1;
            w.modern(&mut r)?;
        } else {
            w.mouse_over = nullable(&mut r)?;
            for _ in 0..r.u8()? {
                w.comparisons.push((r.u8()?, r.u16()?));
            }
            for _ in 0..r.u8()? {
                let mut instructions = Vec::new();
                for _ in 0..r.u16()? {
                    instructions.push(nullable(&mut r)?);
                }
                w.cs1.push(instructions);
            }
            w.legacy(&mut r)?;
        }
        w.trailing = r.bytes(r.remaining())?.to_vec();
        Ok(w)
    }
    fn modern(&mut self, r: &mut Reader<'_>) -> Result<()> {
        match self.kind {
            0 => {
                self.scroll_size = [r.u16()? as i32, r.u16()? as i32];
                self.no_click_through = r.u8()? == 1;
            }
            5 => {
                self.sprites[0] = r.u32()? as i32;
                self.sprite_angle = r.u16()?;
                self.sprite_tiling = r.u8()? == 1;
                self.transparency = r.u8()?;
                self.outline = r.u8()?;
                self.sprite_shadow = r.u32()?;
                self.flip_vertical = r.u8()? == 1;
                self.flip_horizontal = r.u8()? == 1;
            }
            6 => {
                self.models[0] = r.u32()? as i32;
                self.model_offset = [r.u16()? as i16 as i32, r.u16()? as i16 as i32];
                self.model_angles = [r.u16()?, r.u16()?, r.u16()?];
                self.model_zoom = r.u16()?;
                self.sequences[0] = nullable(r)?;
                self.model_orthographic = r.u8()? == 1;
                self.model_extra = r.u16()?;
                if self.size_alignment[0] != 0 {
                    self.model_width_scale = r.u16()?;
                }
                if self.size_alignment[1] != 0 {
                    self.model_height_scale = r.u16()?;
                }
            }
            4 => {
                self.font = nullable(r)?;
                self.text = r.string()?;
                self.line_height = r.u8()?;
                self.text_alignment = [r.u8()?, r.u8()?];
                self.text_shadow = r.u8()? == 1;
                self.colors[0] = r.u32()?;
            }
            3 => {
                self.colors[0] = r.u32()?;
                self.filled = r.u8()? == 1;
                self.transparency = r.u8()?;
            }
            9 => {
                self.line_width = r.u8()?;
                self.colors[0] = r.u32()?;
                self.line_reversed = r.u8()? == 1;
            }
            _ => {}
        }
        self.flags = r.u24()?;
        self.data_text = r.string()?;
        for _ in 0..r.u8()? {
            self.actions.push(r.string()?);
        }
        self.drag_zone = r.u8()? as i32;
        self.drag_threshold = r.u8()? as i32;
        self.scrollbar = r.u8()? == 1;
        self.spell_action = r.string()?;
        for slot in &mut self.listeners {
            let n = r.u8()?;
            if n > 0 {
                let mut args = Vec::new();
                for _ in 0..n {
                    args.push(match r.u8()? {
                        0 => Argument::Int(r.u32()? as i32),
                        1 => Argument::Text(r.string()?),
                        _ => Argument::Null,
                    });
                }
                *slot = Some(args);
            }
        }
        for slot in &mut self.triggers {
            let n = r.u8()?;
            if n > 0 {
                *slot = Some((0..n).map(|_| Ok(r.u32()? as i32)).collect::<Result<_>>()?);
            }
        }
        Ok(())
    }
    fn legacy(&mut self, r: &mut Reader<'_>) -> Result<()> {
        if self.kind == 0 {
            self.scroll_size[1] = r.u16()? as i32;
            self.hidden = r.u8()? == 1;
        }
        if self.kind == 1 {
            r.u16()?;
            r.u8()?;
        }
        if self.kind == 3 {
            self.filled = r.u8()? == 1;
        }
        if self.kind == 4 || self.kind == 1 {
            self.text_alignment = [r.u8()?, r.u8()?];
            self.line_height = r.u8()?;
            self.font = nullable(r)?;
            self.text_shadow = r.u8()? == 1;
        }
        if self.kind == 4 {
            self.text = r.string()?;
            self.alternate_text = r.string()?;
        }
        if matches!(self.kind, 1 | 3 | 4) {
            self.colors[0] = r.u32()?;
        }
        if matches!(self.kind, 3 | 4) {
            self.colors[1] = r.u32()?;
            self.colors[2] = r.u32()?;
            self.colors[3] = r.u32()?;
        }
        if self.kind == 5 {
            self.sprites = [r.u32()? as i32, r.u32()? as i32];
        }
        if self.kind == 6 {
            self.models = [r.u32()? as i32, r.u32()? as i32];
            self.sequences = [nullable(r)?, nullable(r)?];
            self.model_zoom = r.u16()?;
            self.model_angles[0] = r.u16()?;
            self.model_angles[1] = r.u16()?;
        }
        if self.kind == 8 {
            self.text = r.string()?;
        }
        if self.button == 2 {
            self.spell_action = r.string()?;
            self.spell_name = r.string()?;
            self.flags |= (r.u16()? as u32 & 63) << 11;
        }
        if matches!(self.button, 1 | 4 | 5 | 6) {
            self.button_text = r.string()?;
            if self.button_text.is_empty() {
                self.button_text = match self.button {
                    1 => "Ok",
                    4 | 5 => "Select",
                    _ => "Continue",
                }
                .into();
            }
        }
        if matches!(self.button, 1 | 4 | 5) {
            self.flags |= 4194304;
        }
        if self.button == 6 {
            self.flags |= 1;
        }
        Ok(())
    }
    pub fn load_group(cache: &mut DiskCache, id: u32) -> Result<Vec<Self>> {
        if id > 65535 {
            return Err(Error::Invalid("interface group id"));
        }
        cache
            .group(3, id, [0; 4])?
            .into_iter()
            .map(|(file, data)| {
                if file > 65535 {
                    return Err(Error::Invalid("interface file id"));
                }
                Self::decode(id << 16 | file, &data)
                    .map_err(|e| Error::Io(format!("widget {id}:{file}: {e}")))
            })
            .collect()
    }
}
