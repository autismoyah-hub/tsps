//! Robust adapter/smoke pixels from a completed authoritative pick frame.
//! Ordinary mouse picking and movement dispatch do not use this helper.
pub(crate) fn interior(size:[usize;2],tags:&[u64],views:&[u16],matches:impl FnMut(u16,u64)->bool)->Option<([usize;2],u64)> {
    let [width,_]=size;let distance=distances(size,tags,views,matches)?;
    // Retain the previous requirement for a uniform surrounding 3x3 area.
    best(&distance,2,|_|true).map(|i|([i%width,i/width],tags[i]))
}
/// Up to `limit` matching pixels for menu probes, in stable order. The first is
/// exactly `interior`'s pixel; later ones are the deepest remaining pixels at
/// least `SEPARATION` away from every earlier pick, ending with one-pixel-thin
/// regions that `interior` rejects. Work is one distance map plus `limit` scans.
#[cfg_attr(not(feature="private-test-adapter"),allow(dead_code))]
pub(crate) fn candidates(size:[usize;2],tags:&[u64],views:&[u16],matches:impl FnMut(u16,u64)->bool,limit:usize)->Vec<([usize;2],u64)> {
    let [width,_]=size;
    let Some(distance)=distances(size,tags,views,matches) else {return Vec::new();};
    let mut picked:Vec<usize>=Vec::with_capacity(limit);
    while picked.len()<limit {
        let far=|i:usize|picked.iter().all(|&j|(i%width).abs_diff(j%width).max((i/width).abs_diff(j/width))>=SEPARATION);
        let Some(i)=best(&distance,1,far) else {break;};
        picked.push(i);
    }
    picked.into_iter().map(|i|([i%width,i/width],tags[i])).collect()
}
/// Chebyshev spacing between probe pixels, so covered neighbours are skipped.
#[cfg_attr(not(feature="private-test-adapter"),allow(dead_code))]
const SEPARATION:usize=4;
/// Deepest pixel with at least `minimum` depth, lowest index first on ties.
fn best(distance:&[usize],minimum:usize,mut allowed:impl FnMut(usize)->bool)->Option<usize> {
    let mut best:Option<usize>=None;
    for (i,&d) in distance.iter().enumerate() {
        if d>=minimum && best.is_none_or(|b|d>distance[b]) && allowed(i) {best=Some(i);}
    }
    best
}
/// Eight-neighbour distance to a different tag/view or the frame boundary;
/// zero for pixels that do not match.
fn distances(size:[usize;2],tags:&[u64],views:&[u16],mut matches:impl FnMut(u16,u64)->bool)->Option<Vec<usize>> {
    let [width,height]=size;let count=width.checked_mul(height)?;
    if width<3 || height<3 || tags.len()!=count || views.len()!=count {return None;}
    let mut distance=vec![0usize;count];
    let same=|a:usize,b:usize|tags[a]==tags[b] && views[a]==views[b];
    for y in 0..height {for x in 0..width {
        let i=y*width+x;
        if tags[i]==0 || !matches(views[i],tags[i]) {continue;}
        let mut nearest=usize::MAX;
        for (dx,dy) in [(-1isize,0isize),(-1,-1),(0,-1),(1,-1)] {
            let d=x.checked_add_signed(dx).zip(y.checked_add_signed(dy)).filter(|&(a,b)|a<width && b<height)
                .map_or(0,|(a,b)|{let j=b*width+a;if same(i,j){distance[j]}else{0}});
            nearest=nearest.min(d);
        }
        distance[i]=nearest+1;
    }}
    for y in (0..height).rev() {for x in (0..width).rev() {
        let i=y*width+x;if distance[i]==0 {continue;}
        for (dx,dy) in [(1isize,0isize),(-1,1),(0,1),(1,1)] {
            let d=x.checked_add_signed(dx).zip(y.checked_add_signed(dy)).filter(|&(a,b)|a<width && b<height)
                .map_or(0,|(a,b)|{let j=b*width+a;if same(i,j){distance[j]}else{0}});
            distance[i]=distance[i].min(d+1);
        }
    }}
    Some(distance)
}
#[cfg(test)] mod tests {
    use super::*;
    fn brute(size:[usize;2],tags:&[u64],views:&[u16])->Option<([usize;2],u64)> {
        let [w,h]=size;let mut best=None;let mut score=1;
        for y in 0..h {for x in 0..w {
            let i=y*w+x;if tags[i]==0 || views[i]!=0 {continue;}
            let mut radius=0;
            for r in 1..w.min(h) {
                if x<r || y<r || x+r>=w || y+r>=h {break;}
                if !(y-r..=y+r).all(|yy|(x-r..=x+r).all(|xx|tags[yy*w+xx]==tags[i] && views[yy*w+xx]==views[i])) {break;}
                radius=r;
            }
            if radius+1>score {score=radius+1;best=Some(([x,y],tags[i]));}
        }}best
    }
    #[test] fn agrees_with_exhaustive_search_for_holes_labels_and_views() {
        let size=[17,13];let mut random=29u32;
        for _ in 0..120 {
            let mut tags=vec![7;221];let mut views=vec![0;221];
            for i in 0..221 {
                random=random.wrapping_mul(1664525).wrapping_add(1013904223);
                if random%11==0 {tags[i]=0;} else if random%29==0 {tags[i]=11;}
                if random%13==0 {views[i]=1;}
            }
            assert_eq!(interior(size,&tags,&views,|v,_|v==0),brute(size,&tags,&views));
        }
    }
    #[test] fn interior_survives_translation_that_loses_first_three_by_three_pixel() {
        let size=[28,18];let mut tags=vec![0;28*18];let views=vec![0;tags.len()];
        for y in 2..14 {for x in 2..22 {tags[y*28+x]=7;}}
        let (point,tag)=interior(size,&tags,&views,|v,t|v==0 && t==7).unwrap();
        assert_eq!((point,tag),([7,7],7));
        let translated=|[x,y]:[usize;2]|(6..26).contains(&x) && (2..14).contains(&y);
        assert!(!translated([3,3]));assert!(translated(point));
    }
    #[test] fn rejects_thin_absent_malformed_regions_and_keeps_views_separate() {
        assert_eq!(interior([usize::MAX,2],&[],&[],|_,_|true),None);
        assert_eq!(interior([3,3],&[7;8],&[0;9],|_,_|true),None);
        assert_eq!(interior([2,4],&[7;8],&[0;8],|_,_|true),None);
        assert_eq!(interior([5,5],&[7;25],&[1;25],|v,_|v==0),None);
        let tags=vec![7;6*7];let views=(0..7).flat_map(|_|[0,0,0,1,1,1]).collect::<Vec<_>>();
        assert_eq!(interior([6,7],&tags,&views,|v,_|v==0),Some(([1,1],7)));
    }
    #[test] fn first_candidate_is_interior_and_order_is_stable() {
        let size=[17,13];let mut random=31u32;
        for _ in 0..60 {
            let mut tags=vec![7;221];let mut views=vec![0;221];
            for i in 0..221 {
                random=random.wrapping_mul(1664525).wrapping_add(1013904223);
                if random%7==0 {tags[i]=0;} else if random%23==0 {tags[i]=11;}
                if random%17==0 {views[i]=1;}
            }
            let found=candidates(size,&tags,&views,|v,t|v==0 && t==7,8);
            let first=interior(size,&tags,&views,|v,t|v==0 && t==7);
            if let Some(first)=first {assert_eq!(found.first(),Some(&first));}
            assert!(found.len()<=8);
            assert_eq!(found,candidates(size,&tags,&views,|v,t|v==0 && t==7,8));
            for (i,(a,_)) in found.iter().enumerate() {
                let i0=a[1]*17+a[0];assert!(tags[i0]==7 && views[i0]==0);
                for (b,_) in &found[..i] {assert!(a[0].abs_diff(b[0]).max(a[1].abs_diff(b[1]))>=SEPARATION);}
            }
        }
    }
    #[test] fn candidates_stop_at_exact_cap_and_honour_smaller_limits() {
        let size=[64,40];let tags=vec![5;64*40];let views=vec![0;tags.len()];
        assert_eq!(candidates(size,&tags,&views,|_,_|true,8).len(),8);
        assert_eq!(candidates(size,&tags,&views,|_,_|true,3).len(),3);
        assert!(candidates(size,&tags,&views,|_,_|true,0).is_empty());
        assert_eq!(candidates(size,&tags,&views,|_,_|true,8)[0].0,interior(size,&tags,&views,|_,_|true).unwrap().0);
    }
    #[test] fn thin_and_single_pixel_regions_yield_probe_pixels_interior_rejects() {
        // A one-pixel-wide column: no uniform 3x3 interior, but visible pixels exist.
        let size=[9,12];let mut tags=vec![0;108];let views=vec![0;108];
        for y in 1..11 {tags[y*9+4]=3;}
        assert_eq!(interior(size,&tags,&views,|_,t|t==3),None);
        let found=candidates(size,&tags,&views,|_,t|t==3,8);
        assert_eq!(found.iter().map(|(p,_)|*p).collect::<Vec<_>>(),vec![[4,1],[4,5],[4,9]]);
        let mut one=vec![0;108];one[5*9+6]=3;
        assert_eq!(candidates(size,&one,&views,|_,t|t==3,8),vec![([6,5],3)]);
        assert!(candidates(size,&one,&views,|_,t|t==4,8).is_empty());
        assert!(candidates([2,2],&[3;4],&[0;4],|_,_|true,8).is_empty());
        assert!(candidates(size,&one[..107],&views,|_,_|true,8).is_empty());
    }
    #[test] fn covered_interior_still_leaves_separated_visible_candidates() {
        // Target rows 2..14; a panel (other tag) covers its deepest middle band.
        let size=[30,18];let mut tags=vec![0;30*18];let views=vec![0;tags.len()];
        for y in 2..14 {for x in 2..28 {tags[y*30+x]=7;}}
        for y in 2..14 {for x in 10..20 {tags[y*30+x]=9;}}
        let found=candidates(size,&tags,&views,|_,t|t==7,8);
        assert!(found.len()>=2);
        assert!(found.iter().any(|(p,_)|p[0]<10) && found.iter().any(|(p,_)|p[0]>=20));
    }
}
