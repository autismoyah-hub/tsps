//! Password-login input states from GameFriendsChatMember.aq.
use crate::input::Input;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Welcome,
    Token,
    Legal,
    LegalDeclined,
    WorldWarning,
    MembersRequired,
    Credentials,
    Notification,
    Message,
    Privacy,
    DateOfBirth,
    InlineDateOfBirth,
    ConnectingDateOfBirth,
    Connecting,
    Retry,
    InvalidCredentials,
    AccountStatus(u8),
    LauncherRequired,
    Authenticator,
    ConnectingAuthenticator,
}
pub enum Action {
    Login,
    AcceptTerms,
    OpenLegalDocument(&'static str),
    CreateAccount,
    SubmitDateOfBirth,
    SelectWorld,
    Help,
}
pub struct State {
    pub screen: Screen,
    pub token_ready: bool,
    pub required_eula: i32,
    pub accepted_eula: i32,
    pub token_label: String,
    pub username: String,
    pub password: String,
    pub focus: u8,
    pub remember: bool,
    pub hide: bool,
    pub hovered: [bool; 2],
    pub lines: [String; 3],
    pub heading: String,
    pub world_list_loading: bool,
    pub authenticator: String,
    pub trust_computer: bool,
    pub submitted_authenticator: Option<u32>,
    pub date_of_birth: [Option<u8>; 8],
    pub date_cursor: usize,
    pub submitted_date_of_birth: Option<u32>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            screen: Screen::Welcome,
            token_ready: false,
            required_eula: -1,
            accepted_eula: -1,
            token_label: String::new(),
            username: String::new(),
            password: String::new(),
            focus: 0,
            remember: false,
            hide: false,
            hovered: [false; 2],
            lines: Default::default(),
            heading: String::new(),
            world_list_loading: false,
            authenticator: String::new(),
            trust_computer: true,
            submitted_authenticator: None,
            date_of_birth: [None; 8],
            date_cursor: 0,
            submitted_date_of_birth: None,
        }
    }
}
impl State {
    fn start_screen(&self) -> Screen {
        if self.accepted_eula < self.required_eula { Screen::Legal }
        else if self.token_ready { Screen::Token } else { Screen::Welcome }
    }

    /// kh.ao: beta precedes PvP, which may also be high risk.
    pub fn existing_user(&mut self, properties: u32, remembered_username: Option<&str>) {
        let warning = if properties & 33554432 != 0 {
            Some((
                "",
                [
                    "This is a <col=00ffff>Beta<col=ffffff> world.",
                    "Your normal account will not be affected.",
                    "",
                ],
            ))
        } else if properties & 4 != 0 {
            Some((
                "Warning!",
                if properties & 1024 != 0 {
                    [
                        "This is a <col=ffff00>High Risk <col=ff0000>PvP<col=ffffff> world.",
                        "Players can attack each other almost everywhere",
                        "and the Protect Item prayer won't work.",
                    ]
                } else {
                    [
                        "This is a <col=ff0000>PvP<col=ffffff> world.",
                        "Players can attack each other",
                        "almost everywhere.",
                    ]
                },
            ))
        } else if properties & 1024 != 0 {
            Some((
                "Warning!",
                [
                    "This is a <col=ffff00>High Risk<col=ffffff> world.",
                    "The Protect Item prayer will",
                    "not work on this world.",
                ],
            ))
        } else {
            None
        };
        if let Some((heading, lines)) = warning {
            self.screen = Screen::WorldWarning;
            self.heading = heading.into();
            self.lines = lines.map(str::to_owned);
            self.focus = u8::from(self.remember && !self.username.is_empty());
        } else {
            if !self.token_ready { self.restore_username(remembered_username); }
            self.credentials();
        }
    }
    fn restore_username(&mut self, remembered_username: Option<&str>) {
        if self.username.is_empty() {
            self.remember = remembered_username.is_some();
            if let Some(username) = remembered_username {
                self.username = username.into();
            }
        }
        self.focus = u8::from(self.remember && !self.username.is_empty());
    }
    pub fn notification(&mut self, lines: [String; 3]) {
        self.screen = Screen::Notification;
        self.lines = lines;
    }
    pub fn credentials(&mut self) {
        if self.token_ready { self.screen = Screen::Token; return; }
        self.submitted_authenticator = None;
        self.submitted_date_of_birth = None;
        self.screen = Screen::Credentials;
        self.lines = [
            "".into(),
            "Enter your username/email & password.".into(),
            "".into(),
        ];
        self.focus = u8::from(self.remember && !self.username.is_empty());
    }
    fn cancel(&mut self) {
        self.screen = Screen::Welcome;
        self.username.clear();
        self.password.clear();
        self.authenticator.clear();
        self.submitted_authenticator = None;
        self.submitted_date_of_birth = None;
    }
    pub fn submit_date_of_birth(
        &mut self,
        today: crate::date_of_birth::Today,
        noon: impl FnOnce([i32; 3]) -> Option<i64>,
    ) -> Option<Action> {
        match crate::date_of_birth::encode(self.date_of_birth, today, noon) {
            Ok(value) => {
                self.submitted_date_of_birth = Some(value);
                self.screen = Screen::ConnectingDateOfBirth;
                self.lines = ["".into(), "Updating date of birth...".into(), "".into()];
                Some(Action::Login)
            }
            Err(error) => {
                self.lines = error.lines().map(str::to_owned);
                None
            }
        }
    }
    fn submit_authenticator(&mut self) -> Option<Action> {
        if self.authenticator.len() != 6 {
            self.lines = ["".into(), "Please enter a 6-digit PIN.".into(), "".into()];
            return None;
        }
        self.submitted_authenticator = self.authenticator.parse().ok();
        if self.submitted_authenticator.is_none() {
            return None;
        }
        self.authenticator.clear();
        self.lines = ["".into(), "Connecting to server...".into(), "".into()];
        self.screen = Screen::ConnectingAuthenticator;
        Some(Action::Login)
    }
    fn submit_token(&mut self) -> Option<Action> {
        // Original Play selects fresh remembered/password authentication, not a prior PIN or DOB request.
        self.submitted_authenticator = None;
        self.submitted_date_of_birth = None;
        self.lines = ["".into(), "Connecting to server...".into(), "".into()];
        self.screen = Screen::Connecting;
        Some(Action::Login)
    }
    fn submit(&mut self) -> Option<Action> {
        self.username = self.username.trim_matches(|c| c <= '\u{20}').to_owned();
        let message = if self.username.is_empty() {
            "Please enter your username/email address."
        } else if self.password.is_empty() {
            "Please enter your password."
        } else {
            "Connecting to server..."
        };
        self.lines = ["".into(), message.into(), "".into()];
        if !self.username.is_empty() && !self.password.is_empty() {
            self.screen = Screen::Connecting;
            Some(Action::Login)
        } else {
            None
        }
    }
    /// Returns explicit actions for operations owned by the native runtime.
    pub fn input(
        &mut self,
        input: &Input,
        width: i32,
        radio_size: [i32; 2],
        help_width: i32,
    ) -> Option<Action> {
        self.input_with_clipboard(input, width, radio_size, help_width, |_| None)
    }
    pub fn input_with_clipboard(
        &mut self,
        input: &Input,
        width: i32,
        radio_size: [i32; 2],
        help_width: i32,
        clipboard: impl FnMut(usize) -> Option<Vec<u16>>,
    ) -> Option<Action> {
        self.input_with_policy(input, width, radio_size, help_width, false, clipboard)
    }
    pub fn input_with_policy(
        &mut self,
        input: &Input,
        width: i32,
        radio_size: [i32; 2],
        help_width: i32,
        mouse_camera: bool,
        clipboard: impl FnMut(usize) -> Option<Vec<u16>>,
    ) -> Option<Action> {
        self.input_for_world(
            input,
            width,
            radio_size,
            help_width,
            mouse_camera,
            0,
            None,
            clipboard,
        )
    }
    pub fn input_for_world(
        &mut self,
        input: &Input,
        width: i32,
        radio_size: [i32; 2],
        help_width: i32,
        mouse_camera: bool,
        properties: u32,
        remembered_username: Option<&str>,
        mut clipboard: impl FnMut(usize) -> Option<Vec<u16>>,
    ) -> Option<Action> {
        let center = crate::title::center(width);
        let click = (input.pressed_buttons & 1 != 0
            || !mouse_camera && input.pressed_buttons & 4 != 0)
            .then_some(input.press_position);
        let hit = |x: i32, y: i32| {
            click.is_some_and(|p| {
                p[0] >= x - 75 && p[0] <= x + 75 && p[1] >= y - 20 && p[1] <= y + 20
            })
        };
        match self.screen {
            Screen::Legal => {
                if hit(center - 80, 311) {
                    self.accepted_eula = self.required_eula;
                    self.cancel();
                    self.screen = self.start_screen();
                    self.restore_username(remembered_username);
                    return Some(Action::AcceptTerms);
                }
                if hit(center + 80, 311) { self.screen = Screen::LegalDeclined; }
                return None;
            }
            Screen::LegalDeclined => {
                // Original input center is 321 although the drawn button is centered at 311.
                if hit(center, 321) { self.cancel(); self.screen = self.start_screen(); self.restore_username(remembered_username); }
                return None;
            }

            Screen::Token => {
                if click.is_some_and(|p| (center-109..=center+109).contains(&p[0]) && (209..=277).contains(&p[1]))
                    || input.key_events.iter().any(|e| e.0 == 84) {
                    self.existing_user(properties, remembered_username);
                    if self.screen != Screen::WorldWarning { return self.submit_token(); }
                }
                return None;
            }
            Screen::Connecting
            | Screen::ConnectingAuthenticator
            | Screen::ConnectingDateOfBirth => return None,
            Screen::Authenticator => {
                if hit(center - 80, 321) {
                    return self.submit_authenticator();
                }
                if let Some(p) = click {
                    if p[0] >= center - 9 && p[0] <= center + 130 && (263..=296).contains(&p[1]) {
                        self.trust_computer = !self.trust_computer;
                    }
                    if p[0] >= center - 34 && p[0] <= center + 34 && (351..=363).contains(&p[1]) {
                        return Some(Action::Help);
                    }
                }
                if hit(center + 80, 321) {
                    self.cancel();
                }
                for &(key, character) in &input.key_events {
                    if key == 13 {
                        self.cancel();
                    } else {
                        if key == 85 {
                            self.authenticator.pop();
                        }
                        if key == 84 {
                            return self.submit_authenticator();
                        }
                        if (48..=57).contains(&character) && self.authenticator.len() < 6 {
                            self.authenticator
                                .push(char::from_u32(character as u32).unwrap());
                        }
                    }
                }
                return None;
            }
            Screen::AccountStatus(_) => {
                if hit(center, 276) {
                    return Some(Action::Help);
                }
                if hit(center, 326) {
                    self.credentials();
                }
                return None;
            }
            Screen::InlineDateOfBirth => {
                if let Some(p) = click {
                    if (center - 150..=center + 155).contains(&p[0]) && (231..=271).contains(&p[1])
                    {
                        let mut x = center - 150;
                        for i in 0..8 {
                            if p[0] <= x + 30 {
                                self.date_cursor = i;
                                break;
                            }
                            x += if i == 1 || i == 3 { 50 } else { 35 };
                        }
                    }
                }
                if hit(center - 80, 321) {
                    return Some(Action::SubmitDateOfBirth);
                }
                if hit(center + 80, 321) {
                    self.date_of_birth = [None; 8];
                    if !self.token_ready { self.password.clear(); }
                    self.credentials();
                }
                for &(key, character) in &input.key_events {
                    if key == 101 {
                        self.date_of_birth[self.date_cursor] = None;
                    }
                    if key == 85 {
                        if self.date_of_birth[self.date_cursor].is_none() && self.date_cursor > 0 {
                            self.date_cursor -= 1;
                        }
                        self.date_of_birth[self.date_cursor] = None;
                    }
                    if (48..=57).contains(&character) {
                        self.date_of_birth[self.date_cursor] = Some((character - 48) as u8);
                        self.date_cursor = (self.date_cursor + 1).min(7);
                    }
                    if key == 84 {
                        return Some(Action::SubmitDateOfBirth);
                    }
                }
                return None;
            }
            Screen::MembersRequired => {
                if hit(center, 276) {
                    return Some(Action::SelectWorld);
                }
                if hit(center, 326) {
                    if !self.token_ready { self.restore_username(remembered_username); }
                    self.credentials();
                }
                return None;
            }
            Screen::LauncherRequired => {
                if hit(center, 276) {
                    return Some(Action::Help);
                }
                if hit(center, 326) {
                    if !self.token_ready { self.password.clear(); }
                    self.credentials();
                }
                return None;
            }
            Screen::Privacy | Screen::DateOfBirth => {
                if hit(center - 80, 321) {
                    return Some(Action::Help);
                }
                if hit(center + 80, 321) {
                    if !self.token_ready { self.password.clear(); }
                    self.credentials();
                }
                return None;
            }
            Screen::Message => {
                // The original message screen checks only the mouse's Y band.
                if click.is_some_and(|p| (301..=341).contains(&p[1]))
                    || input.key_events.iter().any(|e| matches!(e.0, 13 | 84))
                {
                    if !self.token_ready { self.password.clear(); }
                    self.credentials();
                }
                return None;
            }
            Screen::Retry => {
                if hit(center, 311) || input.key_events.iter().any(|e| matches!(e.0, 13 | 84)) {
                    self.screen = self.start_screen();
                    self.restore_username(remembered_username);
                }
                return None;
            }
            Screen::InvalidCredentials => {
                if hit(center, 276) {
                    self.screen = self.start_screen();
                    self.restore_username(remembered_username);
                }
                if hit(center, 326) {
                    return Some(Action::Help);
                }
                return None;
            }
            Screen::Notification => {
                if hit(center, 301) {
                    self.screen = self.start_screen();
                    self.restore_username(remembered_username);
                }
                return None;
            }
            Screen::WorldWarning => {
                // The original consumes the first Enter/Escape before considering a click.
                let key = input
                    .key_events
                    .iter()
                    .find(|e| matches!(e.0, 13 | 84))
                    .map(|e| e.0);
                if key == Some(84) || key.is_none() && hit(center - 80, 321) {
                    if self.token_ready { return self.submit_token(); }
                    if !self.token_ready { self.restore_username(remembered_username); }
                    self.credentials();
                } else if key == Some(13) || key.is_none() && hit(center + 80, 321) {
                    self.cancel();
                    self.screen = self.start_screen();
                    self.restore_username(remembered_username);
                }
                return None;
            }
            Screen::Welcome => {
                if hit(center - 80, 291) {
                    return Some(Action::CreateAccount);
                }
                if hit(center + 80, 291) || input.key_events.iter().any(|e| e.0 == 84) {
                    self.existing_user(properties, remembered_username);
                }
                return None;
            }
            Screen::Credentials => {}
        }
        if let Some(p) = click {
            if (241..255).contains(&p[1]) {
                self.focus = 0;
            }
            if (256..270).contains(&p[1]) {
                self.focus = 1;
            }
            if p[0] >= center - help_width / 2
                && p[0] <= center + help_width / 2
                && (346..361).contains(&p[1])
            {
                return Some(Action::Help);
            }
        }
        if hit(center - 80, 321) {
            return self.submit();
        }
        if hit(center + 80, 321) {
            self.cancel();
        }
        for (i, x) in [center - 117, center + 24].into_iter().enumerate() {
            let p = click.unwrap_or(input.pointer);
            self.hovered[i] =
                p[0] >= x && p[0] < x + radio_size[0] && p[1] >= 277 && p[1] < 277 + radio_size[1];
            if click.is_some() && self.hovered[i] {
                if i == 0 {
                    self.remember = !self.remember;
                } else {
                    self.hide = !self.hide;
                    if !self.hide {
                        self.username.clear();
                        self.focus = 0;
                    }
                }
            }
        }
        for &(key, character) in &input.key_events {
            if key == 13 {
                self.cancel();
            } else if self.focus == 0 {
                if key == 85 {
                    self.username.pop();
                }
                if key == 84 || key == 80 {
                    self.focus = 1;
                }
                if allowed(character) && self.username.encode_utf16().count() < 320 {
                    self.username
                        .push(char::from_u32(character as u32).unwrap());
                }
            } else {
                if key == 85 && !self.password.is_empty() {
                    self.password.pop();
                } else if key == 84 || key == 80 {
                    self.focus = 0;
                    if key == 84 {
                        return self.submit();
                    }
                }
                if key == 67 && (input.held.contains(&82) || input.held.contains(&87)) {
                    let remaining = 20usize.saturating_sub(self.password.encode_utf16().count());
                    if remaining > 0 {
                        if let Some(text) = clipboard(remaining) {
                            let prefix = &text[..text.len().min(remaining)];
                            if prefix.iter().any(|&ch| !allowed(ch as i32)) {
                                self.screen = Screen::InvalidCredentials;
                                return None;
                            }
                            self.password.push_str(&String::from_utf16_lossy(prefix));
                        }
                    }
                } else if allowed(character) && self.password.encode_utf16().count() < 20 {
                    self.password
                        .push(char::from_u32(character as u32).unwrap());
                }
            }
        }
        None
    }
}
pub fn allowed(character: i32) -> bool {
    char::from_u32(character as u32).is_some_and(|c| "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!\"ï¿½$%^&*()-_=+[{]};:'@#~,<.>/?\\| ".contains(c))
}
