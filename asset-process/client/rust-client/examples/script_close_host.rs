use gielinor_client::{client_state::ClientState,disk_cache::DiskCache,
    interface_packet::SubInterface,script::{Instruction,Operand,Script},
    vm::{Vm,Value,Status},widget::Widget,Result};
use std::rc::Rc;
fn script(ops:&[u16])->Rc<Script> {
    Rc::new(Script{name:None,local_ints:0,local_strings:0,local_longs:0,int_arguments:0,
        string_arguments:0,long_arguments:0,switches:vec![],instructions:ops.iter()
        .map(|&opcode|Instruction{opcode,operand:Operand::Int(0)}).collect()})
}
fn main()->Result<()> {
    let mut client=ClientState::load(DiskCache::open("work/kotlin-world/.data/cache/js5",64<<20)?)?;
    for ending in [21,9999] {
        client.interfaces=Default::default();client.outbound.clear();
        let parent=65534u32<<16;
        client.interfaces.install_group(65534,(0..3).map(|i|Widget::empty(parent+i,0)).collect())?;
        client.interfaces.root=Some(65534);
        for (i,kind) in [0,1,3].into_iter().enumerate() {
            let group=65530+i as u16;
            client.interfaces.install_group(group,vec![Widget::empty((group as u32)<<16,4)])?;
            client.interfaces.children.insert(parent+i as u32,SubInterface{parent:parent+i as u32,group,kind});
            client.interfaces.runtime.get_mut(&(parent+i as u32)).unwrap().listeners.insert(1424,(vec![Value::Int(1)],None));
        }
        client.interfaces.continue_widget=Some(parent);
        let mut vm=Vm::new(script(&[3103,3103,ending]),vec![],vec![],vec![],100)?;
        assert_eq!(vm.run(&mut client,1)?,Status::Yielded);
        assert!(client.outbound.is_empty());assert_eq!(client.interfaces.children.len(),3);
        let result=vm.run(&mut client,100);
        if ending==21 {assert_eq!(result?,Status::Finished);}else{assert!(result.is_err());}
        assert_eq!(client.outbound.len(),1);assert_eq!(client.outbound[0].opcode,95);assert!(client.outbound[0].payload.is_empty());
        assert!(client.interfaces.closing_modal);
        let mut callbacks=0;
        while client.next_interface_event()?.is_some() {
            assert!(client.interfaces.closing_modal);
            let mut callback=Vm::new(script(&[3103,21]),vec![],vec![],vec![],100)?;
            assert_eq!(callback.run(&mut client,100)?,Status::Finished);
            assert_eq!(client.outbound.len(),1,"recursive close packet");callbacks+=1;
        }
        assert_eq!(callbacks,6);assert_eq!(client.interfaces.children.len(),1);
        assert_eq!(client.interfaces.children[&(parent+1)].kind,1);
        assert!(!client.interfaces.closing_modal);assert!(client.interfaces.continue_widget.is_none());
        println!("Client VM close ending{ending}: one packet, {callbacks} guarded callbacks, retained non-modal mount");
    }
    Ok(())
}
