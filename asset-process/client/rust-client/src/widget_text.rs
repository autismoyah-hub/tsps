//! ga.ae item labels use GameItemComposition.aq, not inventory-icon quantities.
pub fn quantity(count: i32) -> String {
    let mut text = count.to_string();
    let mut at = text.len() as i32 - 3;
    while at > 0 {
        text.insert(at as usize, ',');
        at -= 3;
    }
    if text.len() > 9 {
        format!(" <col=ff80>{}M  ({text})</col>", &text[..text.len() - 8])
    } else if text.len() > 6 {
        format!(" <col=ffffff>{}K  ({text})</col>", &text[..text.len() - 4])
    } else {
        format!(" <col=ffff00>{text}</col>")
    }
}
pub fn item(name: &str, stackable: i32, count: i32) -> String {
    if (stackable == 1 || count != 1) && count != -1 {
        format!("<col=ff9040>{name}</col> x{}", quantity(count))
    } else {
        name.to_owned()
    }
}
