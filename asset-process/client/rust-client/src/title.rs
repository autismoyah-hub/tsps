//! Cache-backed title layout recovered from zf.ae, kb.ap and nf.au.
use crate::{
    disk_cache::DiskCache,
    font::{Font, Style},
    raster::Raster,
    sprite::SpriteSheet,
    sprite_pixels::Pixels,
    Error, Result,
};
pub struct Assets {
    fire: Option<crate::title_fire::Fire>,
    mute: Option<[Pixels; 2]>,
    world_button: Option<Pixels>,
    background: Pixels,
    mirrored: Pixels,
    logo: Pixels,
    panel: Pixels,
    button: Pixels,
    token_button: Option<[Pixels; 2]>,
    bold: Font,
    form: Option<FormAssets>,
    regular: Option<Font>,
}
struct FormAssets {
    small: Font,
    radio: [Pixels; 4],
}
pub const LEGAL_LINES: [&str; 4] = [
    "Before using this app, please read and accept our",
    "<col=ffd200>terms of use</col>, <col=ffd200>privacy policy</col>, and <col=ffd200>end user licence</col>",
    "<col=ffd200>agreement (EULA)</col>.",
    "By accepting, you agree to these documents.",
];
pub fn logo_name(properties: u32) -> &'static str {
    if properties & 0x2000_0000 != 0 {
        "logo_deadman_mode"
    } else if properties & 0x4000_0000 != 0 {
        "logo_seasonal_mode"
    } else if properties & 256 != 0 {
        "logo_speedrunning"
    } else if properties & 2048 != 0 {
        "logo_ugc_world"
    } else {
        "logo"
    }
}
pub fn center(width: i32) -> i32 {
    (width - 765) / 2 + 382
}
impl Assets {
    pub fn load(cache: &mut DiskCache, properties: u32) -> Result<Self> {
        let font = cache
            .reference(13)?
            .groups
            .iter()
            .find(|g| g.name_hash == Some(crate::disk_cache::name_hash("b12_full")))
            .ok_or(Error::Invalid("title font"))?
            .id;
        let mut assets = Self::decode(
            &cache.named_file(10, "title.jpg", "")?,
            [
                &cache.named_file(8, logo_name(properties), "")?,
                &cache.named_file(8, "titlebox", "")?,
                &cache.named_file(8, "titlebutton", "")?,
            ],
            [
                &cache.file(13, font, 0, [0; 4])?,
                &cache.file(8, font, 0, [0; 4])?,
            ],
        )?;
        let small = cache
            .reference(13)?
            .groups
            .iter()
            .find(|g| g.name_hash == Some(crate::disk_cache::name_hash("p11_full")))
            .ok_or(Error::Invalid("title small font"))?
            .id;
        assets.install_form(
            [
                &cache.file(13, small, 0, [0; 4])?,
                &cache.file(8, small, 0, [0; 4])?,
            ],
            [
                &cache.named_file(8, "options_radio_buttons,0", "")?,
                &cache.named_file(8, "options_radio_buttons,4", "")?,
                &cache.named_file(8, "options_radio_buttons,2", "")?,
                &cache.named_file(8, "options_radio_buttons,6", "")?,
            ],
        )?;
        let regular = cache
            .reference(13)?
            .groups
            .iter()
            .find(|g| g.name_hash == Some(crate::disk_cache::name_hash("p12_full")))
            .ok_or(Error::Invalid("title regular font"))?
            .id;
        assets.install_regular([
            &cache.file(13, regular, 0, [0; 4])?,
            &cache.file(8, regular, 0, [0; 4])?,
        ])?;
        let mute = SpriteSheet::decode(&cache.named_file(8, "title_mute", "")?, 1 << 20)?;
        assets.mute = Some([Pixels::from_sheet(&mute, 0)?, Pixels::from_sheet(&mute, 1)?]);
        assets.world_button = Some(decode_sprite(&cache.named_file(8, "sl_button", "")?)?);
        assets.install_token([&cache.named_file(8, "titlebutton_large", "")?, &cache.named_file(8, "play_now_text", "")?])?;
        let mut seed = [0u8; 8];
        getrandom::getrandom(&mut seed).map_err(|e| Error::Io(e.to_string()))?;
        assets.fire = Some(crate::title_fire::Fire::new(
            SpriteSheet::decode(&cache.named_file(8, "runes", "")?, 1 << 20)?,
            crate::title_fire::colors(&cache.file(17, 3, 0, [0; 4])?)?,
            u64::from_le_bytes(seed),
        )?);
        Ok(assets)
    }
    pub fn decode(background: &[u8], sprites: [&[u8]; 3], font: [&[u8]; 2]) -> Result<Self> {
        #[cfg(not(feature = "offscreen"))]
        let background = crate::image_decode::decode(background, 1 << 20)?;
        #[cfg(feature = "offscreen")]
        // #233: offscreen has no image decoder, so the background is absent: an empty image that
        // draws nothing. Every other title asset decodes as usual.
        let background = crate::image_decode::decode(background, 1 << 20)
            .unwrap_or(Pixels { canvas: [0, 0], offset: [0, 0], size: [0, 0], pixels: Vec::new() });
        let mut mirrored = background.clone();
        mirrored.flip(true, false);
        Ok(Self {
            token_button: None,
            world_button: None,
            fire: None,
            mute: None,
            background,
            mirrored,
            logo: decode_sprite(sprites[0])?,
            panel: decode_sprite(sprites[1])?,
            button: decode_sprite(sprites[2])?,
            bold: Font::decode(font[0], font[1])?,
            form: None,
            regular: None,
        })
    }
    pub fn install_token(&mut self, sprites: [&[u8]; 2]) -> Result<()> {
        self.token_button = Some([decode_sprite(sprites[0])?, decode_sprite(sprites[1])?]);
        Ok(())
    }
    pub fn install_regular(&mut self, font: [&[u8]; 2]) -> Result<()> {
        self.regular = Some(Font::decode(font[0], font[1])?);
        Ok(())
    }
    pub fn install_form(&mut self, font: [&[u8]; 2], radio: [&[u8]; 4]) -> Result<()> {
        self.form = Some(FormAssets {
            small: Font::decode(font[0], font[1])?,
            radio: [
                decode_sprite(radio[0])?,
                decode_sprite(radio[1])?,
                decode_sprite(radio[2])?,
                decode_sprite(radio[3])?,
            ],
        });
        Ok(())
    }
    pub fn legal_link(&self, width: i32, point: [i32; 2]) -> Option<&'static str> {
        let font=self.regular.as_ref()?;
        for (line,start,len,y,url) in [
            (LEGAL_LINES[1],0,30,233,"https://legal.jagex.com/docs/terms"),
            (LEGAL_LINES[1],32,32,233,"https://legal.jagex.com/docs/policies/privacy"),
            (LEGAL_LINES[1],70,34,233,"https://legal.jagex.com/docs/terms/eula"),
            (LEGAL_LINES[2],0,34,250,"https://legal.jagex.com/docs/terms/eula"),
        ] {
            let x=center(width)-font.metrics.width(line,&[])/2+font.metrics.width(&line[..start],&[]);
            let w=font.metrics.width(&line[start..start+len],&[]);
            let top=y-font.max_ascent;
            if (x..x+w).contains(&point[0]) && (top..y+font.max_descent).contains(&point[1]) { return Some(url); }
        }
        None
    }
    pub fn input_dimensions(&self) -> Result<([i32; 2], i32)> {
        let form = self
            .form
            .as_ref()
            .ok_or(Error::Invalid("title form assets missing"))?;
        Ok((
            form.radio[0].size.map(|v| v as i32),
            form.small.metrics.width("Can't login? Click here.", &[]),
        ))
    }
    pub fn set_world_logo(&mut self, cache: &mut DiskCache, properties: u32) -> Result<()> {
        self.logo = decode_sprite(&cache.named_file(8, logo_name(properties), "")?)?;
        Ok(())
    }
    pub fn draw_world_button(&self, raster: &mut Raster, world: i32, loading: bool) -> Result<()> {
        if let (Some(button), Some(form)) = (&self.world_button, &self.form) {
            let x = (raster.width() as i32 - 765) / 2 + 5;
            button.draw(raster, x, 463, 256)?;
            for (font, label, y) in [(&self.bold, format!("World {world}"), 478), (&form.small, if loading {"Loading..."} else {"Click to switch"}.into(), 492)] {
                font.draw(raster, &label, x + 50 - font.metrics.width(&label, &[]) / 2, y, Style::default(), &[])?;
            }
        }
        Ok(())
    }
    pub fn draw_mute(&self, raster: &mut Raster, disabled: bool) -> Result<()> {
        if let Some(mute) = &self.mute {
            mute[usize::from(disabled)].draw(
                raster,
                (raster.width() as i32 - 765) / 2 + 725,
                463,
                256,
            )?;
        }
        Ok(())
    }
    pub fn draw_fire(&mut self, raster: &mut Raster, cycle: i32) {
        if let Some(fire) = &mut self.fire {
            fire.advance(cycle);
            let left = (raster.width() as i32 - 765) / 2;
            fire.draw(raster, left - 22);
            fire.draw(raster, left + 659);
        }
    }
    pub fn draw(
        &self,
        raster: &mut Raster,
        state: &crate::title_input::State,
        cycle: i32,
    ) -> Result<()> {
        use crate::title_input::Screen;
        if state.screen == Screen::Notification {
            return self.notification(raster, [&state.lines[0], &state.lines[1], &state.lines[2]]);
        }
        self.background(raster)?;
        let x = center(raster.width() as i32);
        if matches!(state.screen, Screen::LauncherRequired | Screen::MembersRequired) {
            self.text(raster, &state.lines[0], x, 201, 0xffff00)?;
            let form=self.form.as_ref().ok_or(Error::Invalid("title form assets missing"))?;
            let regular=self.regular.as_ref().ok_or(Error::Invalid("title regular font missing"))?;
            let style=Style{color:0xffff00,shadow:Some(0),alpha:256};
            for (i,line) in state.lines[1..].iter().enumerate() {
                form.small.draw(raster,line,x-form.small.metrics.width(line,&[])/2,221+i as i32*20,style,&[])?;
            }
            for (label,y) in [(if state.screen == Screen::MembersRequired { if state.world_list_loading { "Loading..." } else { "Switch World" } } else { "Download Launcher" },276),("Back",326)] {
                self.button.draw(raster,x-73,y-20,256)?;
                regular.draw(raster,label,x-regular.metrics.width(label,&[])/2,y+5,Style{color:0xffffff,..style},&[])?;
            }
            return Ok(());
        }
        if let Screen::AccountStatus(reason) = state.screen {
            let lines = match reason {
                0 => [
                    "Your account has been involved",
                    "in serious rule breaking.",
                    "",
                ],
                1 => [
                    "Your account has been locked due to",
                    "suspicious activity.",
                    "Please recover your account.",
                ],
                _ => [
                    "The unpaid balance on your account needs",
                    "to be resolved before you can play.",
                    "Please visit the support page for assistance.",
                ],
            };
            for (i, line) in lines.iter().enumerate() {
                self.text(raster, line, x, 201 + i as i32 * 20, 0xffff00)?;
            }
            self.button(
                raster,
                if reason == 1 {
                    "Recover Account"
                } else {
                    "View Appeal Options"
                },
                x,
                276,
            )?;
            return self.button(raster, "Back", x, 326);
        }
        if matches!(state.screen, Screen::InlineDateOfBirth | Screen::ConnectingDateOfBirth) {
            for (i,line) in state.lines.iter().enumerate() {self.text(raster,line,x,201+i as i32*15,0xffff00)?;}
            let style=Style{color:0xffffff,shadow:Some(0),alpha:256};
            let mut left=x-150;
            for (i,digit) in state.date_of_birth.iter().enumerate() {
                self.button.draw_scaled(raster,[left,241,30,40],256)?;
                let mut text=digit.map_or_else(String::new,|d|d.to_string());
                if i==state.date_cursor && cycle%40<20 {text.push_str("<col=ffff00>|");}
                self.bold.draw(raster,&text,left+10,268,style,&[])?;
                left+=if i==1 || i==3 {50}else{35};
                if i==1 || i==3 {self.bold.draw(raster,"/",left-13,268,style,&[])?;}
            }
            self.button(raster,"Submit",x-80,321)?;
            return self.button(raster,"Cancel",x+80,321);
        }
        if matches!(state.screen, Screen::Privacy | Screen::DateOfBirth) {
            let (heading, lines, label) = if state.screen == Screen::Privacy {
                (
                    "Sorry, but your account is not eligible to play.",
                    [
                        "For more information, please take a look at",
                        "our privacy policy.",
                    ],
                    "Privacy Policy",
                )
            } else {
                (
                    "Your date of birth isn't set.",
                    [
                        "Please verify your account status by",
                        "setting your date of birth.",
                    ],
                    "Set Date of Birth",
                )
            };
            self.text(raster, heading, x, 216, 0xffff00)?;
            let font = self
                .regular
                .as_ref()
                .ok_or(Error::Invalid("title regular font missing"))?;
            for (i, line) in lines.iter().enumerate() {
                font.draw(
                    raster,
                    line,
                    x - font.metrics.width(line, &[]) / 2,
                    231 + i as i32 * 15,
                    Style {
                        color: 0xffff00,
                        shadow: Some(0),
                        alpha: 256,
                    },
                    &[],
                )?;
            }
            self.button(raster, label, x - 80, 321)?;
            return self.button(raster, "Back", x + 80, 321);
        }
        if state.screen == Screen::Message {
            for (i, line) in state.lines.iter().enumerate() {
                self.text(raster, line, x, 201 + i as i32 * 15, 0xffff00)?;
            }
            return self.button(raster, "Back", x, 321);
        }
        if state.screen == Screen::Retry {
            for (i, line) in state.lines.iter().enumerate() {
                self.text(raster, line, x, 221 + i as i32 * 25, 0xffff00)?;
            }
            return self.button(raster, "Try again", x, 311);
        }
        if state.screen == Screen::Authenticator {
            let form = self
                .form
                .as_ref()
                .ok_or(Error::Invalid("title form assets missing"))?;
            self.text(raster, "Authenticator", x, 201, 0xffff00)?;
            for (i, line) in state.lines.iter().enumerate() {
                self.text(raster, line, x, 236 + i as i32 * 15, 0xffffff)?;
            }
            let style = Style {
                color: 0xffffff,
                shadow: Some(0),
                alpha: 256,
            };
            let cursor = if cycle % 40 < 20 { "<col=ffff00>|" } else { "" };
            self.bold.draw(
                raster,
                &format!("PIN: {}{cursor}", "*".repeat(state.authenticator.len())),
                x - 108,
                281,
                style,
                &[],
            )?;
            for (line, y) in [("Trust this computer", 273), ("for 30 days: ", 288)] {
                self.bold.draw(
                    raster,
                    line,
                    x - 9,
                    y,
                    Style {
                        color: 0xffff00,
                        ..style
                    },
                    &[],
                )?;
            }
            form.radio[usize::from(state.trust_computer) * 2].draw(
                raster,
                x - 9 + self.bold.metrics.width("for 30 days: ", &[]) + 15,
                288 - self.bold.metrics.ascent,
                256,
            )?;
            self.button(raster, "Continue", x - 80, 321)?;
            self.button(raster, "Cancel", x + 80, 321)?;
            let help = "Can't login? Click here.";
            form.small.draw(
                raster,
                help,
                x - form.small.metrics.width(help, &[]) / 2,
                357,
                style,
                &[],
            )?;
            return Ok(());
        }
        if state.screen == Screen::InvalidCredentials {
            let form = self
                .form
                .as_ref()
                .ok_or(Error::Invalid("title form assets missing"))?;
            let regular = self
                .regular
                .as_ref()
                .ok_or(Error::Invalid("title regular font missing"))?;
            self.text(raster, "Incorrect username or password.", x, 201, 0xffff00)?;
            let style = Style {
                color: 0xffff00,
                shadow: Some(0),
                alpha: 256,
            };
            for (line, y) in [("If you have upgraded to a Jagex Account, you", 221), ("need to log in using the <u=ffd200><col=ffd200>Jagex Launcher</col></u> instead.", 241)] {
                form.small.draw(raster, line, x - form.small.metrics.width(line, &[]) / 2, y, style, &[])?;
            }
            for (label, y) in [("Try again", 276), ("Forgotten password?", 326)] {
                self.button.draw(raster, x - 73, y - 20, 256)?;
                regular.draw(
                    raster,
                    label,
                    x - regular.metrics.width(label, &[]) / 2,
                    y + 5,
                    Style {
                        color: 0xffffff,
                        ..style
                    },
                    &[],
                )?;
            }
            return Ok(());
        }
        if state.screen == Screen::WorldWarning {
            self.text(raster, &state.heading, x, 201, 0xffff00)?;
            for (i, line) in state.lines.iter().enumerate() {
                self.text(raster, line, x, 236 + i as i32 * 15, 0xffffff)?;
            }
            for (label, bx) in [("Continue", x - 80), ("Cancel", x + 80)] {
                self.button.draw(raster, bx - 73, 301, 256)?;
                self.text(raster, label, bx, 326, 0xffffff)?;
            }
            return Ok(());
        }
        if matches!(state.screen, Screen::Legal | Screen::LegalDeclined) {
            let font=self.regular.as_ref().ok_or(Error::Invalid("title regular font missing"))?;
            let lines: &[&str]=if state.screen==Screen::Legal { &LEGAL_LINES } else { &["You must accept our terms of use, privacy policy,", "and end user licence agreement to continue."] };
            for (i,line) in lines.iter().enumerate() {
                let y=if state.screen==Screen::Legal {216+i as i32*17} else {231+i as i32*20};
                font.draw(raster,line,x-font.metrics.width(line,&[])/2,y,Style{color:0xffffff,shadow:Some(0),alpha:256},&[])?;
            }
            if state.screen==Screen::Legal {
                self.button(raster,"Accept",x-80,311)?;
                return self.button(raster,"Decline",x+80,311);
            }
            return self.button(raster,"Back",x,311);
        }
        if state.screen == Screen::Token {
            self.text(raster, "Welcome to RuneScape", x, 209, 0xffff00)?;
            let sprites = self.token_button.as_ref().ok_or(Error::Invalid("token title assets missing"))?;
            sprites[0].draw(raster, x - 109, 229, 256)?;
            sprites[1].draw(raster, x - 48, if state.token_label.is_empty() {247} else {234}, 256)?;
            if !state.token_label.is_empty() { self.text(raster, &state.token_label, x, 282, 0xffff00)?; }
            return Ok(());
        }
        if state.screen == Screen::Welcome {
            self.text(raster, "Welcome to RuneScape", x, 251, 0xffff00)?;
            for (label, bx) in [("New User", x - 80), ("Existing User", x + 80)] {
                self.button.draw(raster, bx - 73, 271, 256)?;
                self.bold.draw_paragraph(
                    raster,
                    label,
                    [bx - 73, 271, 144, 40],
                    Style {
                        color: 0xffffff,
                        shadow: Some(0),
                        alpha: 256,
                    },
                    [1, 1],
                    0,
                    &[],
                )?;
            }
            return Ok(());
        }
        for (i, line) in state.lines.iter().enumerate() {
            self.text(raster, line, x, 201 + i as i32 * 15, 0xffff00)?;
        }
        if state.screen == Screen::ConnectingAuthenticator || state.screen == Screen::Connecting && state.token_ready {
            return Ok(());
        }
        let style = Style {
            color: 0xffffff,
            shadow: Some(0),
            alpha: 256,
        };
        self.bold
            .draw(raster, "Login: ", x - 110, 253, style, &[])?;
        let mut username = if state.hide {
            "*".repeat(state.username.encode_utf16().count())
        } else {
            state.username.clone()
        };
        while self.bold.metrics.width(&username, &[]) > 200 {
            if state.screen == Screen::Connecting {
                username.pop();
            } else {
                username.remove(0);
            }
        }
        let mut username = escape(&username);
        let mut password = "*".repeat(state.password.encode_utf16().count());
        while self.bold.metrics.width(&password, &[]) > 200 {
            password.remove(0);
        }
        if state.screen == Screen::Credentials && cycle % 40 < 20 {
            if state.focus == 0 {
                username.push_str("<col=ffff00>|");
            } else {
                password.push_str("<col=ffff00>|");
            }
        }
        self.bold.draw(raster, &username, x - 70, 253, style, &[])?;
        self.bold.draw(
            raster,
            &format!("Password: {password}"),
            x - 108,
            268,
            style,
            &[],
        )?;
        if state.screen == Screen::Connecting {
            return Ok(());
        }
        let form = self
            .form
            .as_ref()
            .ok_or(Error::Invalid("title form assets missing"))?;
        for (index, (selected, rx, label)) in [
            (state.remember, x - 117, "Remember username"),
            (state.hide, x + 24, "Hide username"),
        ]
        .into_iter()
        .enumerate()
        {
            let radio = &form.radio[usize::from(selected) * 2 + usize::from(state.hovered[index])];
            radio.draw(raster, rx, 277, 256)?;
            form.small.draw(
                raster,
                label,
                rx + radio.size[0] as i32 + 5,
                290,
                Style {
                    color: 0xffff00,
                    ..style
                },
                &[],
            )?;
        }
        self.button(raster, "Login", x - 80, 321)?;
        self.button(raster, "Cancel", x + 80, 321)?;
        let help = "Can't login? Click here.";
        form.small.draw(
            raster,
            help,
            x - form.small.metrics.width(help, &[]) / 2,
            357,
            style,
            &[],
        )?;
        Ok(())
    }
    pub fn background(&self, raster: &mut Raster) -> Result<()> {
        let left = (raster.width() as i32 - 765) / 2;
        self.background.draw(raster, left, 0, 256)?;
        self.mirrored.draw(raster, left + 382, 0, 256)?;
        self.logo
            .draw(raster, left + 382 - self.logo.size[0] as i32 / 2, 18, 256)?;
        self.panel.draw(raster, left + 202, 171, 256)
    }
    fn text(&self, raster: &mut Raster, text: &str, x: i32, y: i32, color: u32) -> Result<()> {
        self.bold
            .draw(
                raster,
                text,
                x - self.bold.metrics.width(text, &[]) / 2,
                y,
                Style {
                    color,
                    shadow: Some(0),
                    alpha: 256,
                },
                &[],
            )
            .map(|_| ())
    }
    fn button(&self, raster: &mut Raster, text: &str, x: i32, y: i32) -> Result<()> {
        self.button.draw(raster, x - 73, y - 20, 256)?;
        self.text(raster, text, x, y + 5, 0xffffff)
    }
    /// Login index 24: three response lines and an acknowledgement button.
    pub fn notification(&self, raster: &mut Raster, lines: [&str; 3]) -> Result<()> {
        self.background(raster)?;
        let x = center(raster.width() as i32);
        for (index, line) in lines.iter().enumerate() {
            self.text(raster, line, x, 221 + index as i32 * 15, 0xffffff)?;
        }
        self.button(raster, "Ok", x, 301)
    }
}
fn escape(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        match c {
            '<' => out.push_str("<lt>"),
            '>' => out.push_str("<gt>"),
            _ => out.push(c),
        }
    }
    out
}
fn decode_sprite(bytes: &[u8]) -> Result<Pixels> {
    Pixels::from_sheet(&SpriteSheet::decode(bytes, 1 << 20)?, 0)
}
