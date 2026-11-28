use gielinor_client::{buffer::Reader,client_state::ClientState,disk_cache::DiskCache,
    vm::{Host,Stacks,Object,Control,Value},widget::Widget,input::Input,Result};
fn text(r:&mut Reader<'_>)->Result<String> {let n=r.u32()? as usize;Ok(std::str::from_utf8(r.bytes(n)?).unwrap().to_owned())}
fn main()->Result<()> {
    let mut client=ClientState::load(DiskCache::open("work/kotlin-world/.data/cache/js5",64<<20)?)?;
    let mut r=Reader::new(include_bytes!("../tests/fixtures/dialog-resume-jvm.bin"));
    let cases=r.u32()?;
    for case in 0..cases {
        let opcode=r.u32()? as u16;let text=text(&mut r)?;let success=r.u8()?!=0;
        let mut stack=Stacks::default();stack.push_object(Object::String(text))?;
        client.outbound.clear();let result=client.extended(opcode,false,&mut stack);
        assert!(stack.objects().is_empty(),"opcode{opcode} dispatch: {result:?}");
        if !success {assert!(result.is_err());assert!(client.outbound.is_empty());continue;}
        assert_eq!(result?,Control::Continue);let expected_opcode=r.u32()? as u16;let n=r.u32()? as usize;let payload=r.bytes(n)?;
        assert_eq!(client.outbound.len(),1,"case{case}");assert_eq!(client.outbound[0].opcode,expected_opcode);assert_eq!(client.outbound[0].payload,payload);
    }
    assert_eq!(r.remaining(),0);client.outbound.clear();
    let owner=65534u32<<16;let mut widget=Widget::empty(owner,0);widget.if3=true;widget.raw_size=[100,100];
    client.interfaces.install_group(65534,vec![widget])?;client.interfaces.root=Some(65534);
    client.interfaces.runtime.get_mut(&owner).unwrap().listeners.insert(1418,(vec![Value::Int(1)],None));
    let layout=client.interfaces.layout([765,503])?;
    let mut r=Reader::new(include_bytes!("../tests/fixtures/script-message-jvm.bin"));let messages=r.u32()?;
    for case in 0..messages {
        let input=text(&mut r)?;client.cycle=r.u32()? as i32;let uid=r.u32()? as i32;let cycle=r.u32()? as i32;
        let name=text(&mut r)?;let value=text(&mut r)?;assert_ne!(r.u8()?,0);let count=r.u32()? as usize;assert_eq!(r.u32()?,case+1);
        let mut stack=Stacks::default();stack.push_object(Object::String(input))?;
        assert_eq!(client.extended(3100,false,&mut stack)?,Control::Continue);assert!(stack.objects().is_empty());assert!(client.outbound.is_empty());
        let line=client.chat.line(0,0).unwrap();assert_eq!((line.uid,line.cycle),(uid,cycle));assert_eq!(line.name,name);assert_eq!(line.text,value);assert!(line.prefix.is_empty());
        assert_eq!(client.chat.count(0),count);assert_eq!(client.game_messages.len(),count);assert_eq!(client.game_messages.back().unwrap().text,value);
        let notifications=client.interface_events.collect(&client.interfaces,&layout,&Input::default(),false,false);
        assert_eq!(notifications.len(),1,"chat transmit case{case}");assert_eq!(notifications[0].widget,owner);
        assert!(client.interface_events.collect(&client.interfaces,&layout,&Input::default(),false,false).is_empty());
    }
    assert_eq!(r.remaining(),0);
    println!("Actual ClientState: {cases} original dialog responses and {messages} script messages, history eviction and chat transmit notifications");
    Ok(())
}
