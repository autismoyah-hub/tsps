//! Optional private alignment evidence from an already admitted authored mesh.
use gielinor_client::{render3d::Triangle,Error,Result};
use std::fmt::Write;
const MAX_ROWS:usize=100_000;
const MAX_BYTES:usize=16<<20;
pub fn export(triangles:&[Triangle],center:[f32;3])->Result<(String,usize)> {
    if center.iter().any(|v|!v.is_finite()){return Err(Error::Invalid("triangle inspection center"));}
    let mut csv=String::from("kind,object_id,tag,view,first_lit_color,x0,y0,z0,x1,y1,z1,x2,y2,z2\n");
    let mut rows=0;
    for t in triangles {
        // Exact source all-vertices/strict-boundary selection, in native scene units.
        if !t.vertices.iter().all(|p|(p[0]-center[0]).abs()<550. && (p[2]-center[2]).abs()<550. && (p[1]-center[1]).abs()<12.) {continue;}
        if rows==MAX_ROWS{return Err(Error::Invalid("triangle inspection row budget"));}
        let object=(t.tag>>63==0 && t.tag>>32!=0).then(||(t.tag>>32)-1);
        let kind=if t.tag==0 {"terrain"}else if object.is_some(){"object"}else{"other"};
        write!(csv,"{kind},").unwrap();if let Some(id)=object{write!(csv,"{id}").unwrap();}
        write!(csv,",{},{},{}",t.tag,t.view,t.colors[0]).unwrap();
        for vertex in t.vertices {for value in vertex {write!(csv,",{value}").unwrap();}}
        csv.push('\n');rows+=1;
        if csv.len()>MAX_BYTES{return Err(Error::Invalid("triangle inspection byte budget"));}
    }
    Ok((csv,rows))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn triangle(tag:u64)->Triangle {
        let v=[[0.,-240.,0.],[20.,-240.,0.],[0.,-240.,20.]];
        let mut t=Triangle::selection_only(gielinor_client::model_bounds::Bounds::new(&v,[0;3]),tag);
        t.vertices=v;t.model_bounds=None;t.alpha=0;t.colors=[123;3];t
    }
    #[test]
    fn strict_source_bounds_require_all_vertices_and_preserve_order() {
        let a=triangle(7u64<<32);let mut edge=a;edge.vertices[2][0]=550.;
        let mut high=a;high.vertices[0][1]=-228.;let mut bad=a;bad.vertices[0][2]=f32::NAN;
        let mut inside=a;inside.vertices[2][0]=549.5;inside.view=9;
        let (csv,rows)=export(&[a,edge,high,bad,inside],[0.,-240.,0.]).unwrap();
        assert_eq!(rows,2);let lines=csv.lines().collect::<Vec<_>>();assert_eq!(lines.len(),3);
        assert!(lines[1].starts_with("object,6,30064771072,0,123,"));
        assert!(lines[2].starts_with("object,6,30064771072,9,123,"));
        assert!(export(&[a],[f32::NAN,0.,0.]).is_err());
    }
    #[test]
    fn zero_and_nonobject_tags_do_not_invent_wrapped_object_ids() {
        let (csv,rows)=export(&[triangle(0),triangle(1),triangle(1u64<<63)],[0.,-240.,0.]).unwrap();
        assert_eq!(rows,3);let lines=csv.lines().collect::<Vec<_>>();
        assert!(lines[1].starts_with("terrain,,0,0,"));assert!(lines[2].starts_with("other,,1,0,"));
        assert!(lines[3].starts_with("other,,9223372036854775808,0,"));
        assert!(!csv.contains("18446744073709551615"));
    }
    #[test]
    fn oversized_local_exports_fail_before_output_publication() {
        assert!(export(&vec![triangle(0);MAX_ROWS+1],[0.,-240.,0.]).is_err());
    }
}
