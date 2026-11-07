//! The settings window, on `rea_launcher_ui`.
//!
//! [`draw`] is the whole window as a function of [`Window`], so the
//! headless tests below drive the real widgets (egui_kittest) on the host;
//! [`show`] wraps it in eframe on Linux and Windows.
//!
//! Layout: a tropical panel on the left, the title and status, the Play
//! tab (ROM and look cards), the Picture tab (the emulator options) and
//! About, then the footer with the Play plate.

use std::path::{Path, PathBuf};

use rea_launcher::cfg::{OptionSpec, spec};
use rea_launcher_ui::egui::{self, Align2, Rect, RichText, TextureHandle, pos2, vec2};
use rea_launcher_ui::spec::{mark_rendered, spec_path_row, spec_row};
use rea_launcher_ui::theme::{body, condensed, tint};
use rea_launcher_ui::widgets::{self, Crop, Fade, LookCard};
use rea_launcher_ui::window::Layout;

use crate::config::{Look, OPTIONS};
use crate::controls::{Controls, PadLayout, SnesButton};
use crate::edition;
use crate::form::Form;
use crate::palette::{ACCENT, BASE, ERROR, INK, MUTED, PANEL};

/// Width of the illustrated strip.
pub const PHOTO_W: f32 = 280.0;

/// The window is a little taller than the SDK default so the five look
/// cards fit without scrolling on first open.
pub const WINDOW_SIZE: egui::Vec2 = egui::vec2(960.0, 740.0);

/// No image is bundled: the game's own art is copyrighted and the window
/// uses an original tropical gradient instead (docs/STANDARDS.md).
pub const HERO_JPEG: Option<&[u8]> = None;
pub const HERO_CREDIT: &str = "";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tab {
    Play,
    Picture,
    Controls,
    About,
}

impl Tab {
    pub const NAMES: [(Tab, &'static str); 4] = [
        (Tab::Play, "Play"),
        (Tab::Picture, "Picture"),
        (Tab::Controls, "Controls"),
        (Tab::About, "About"),
    ];

    /// The next (`by` = 1) or previous (-1) tab, wrapping (pad LB/RB).
    pub fn step(self, by: isize) -> Tab {
        let at = Tab::NAMES
            .iter()
            .position(|(tab, _)| *tab == self)
            .unwrap_or(0) as isize;
        let len = Tab::NAMES.len() as isize;
        Tab::NAMES[(at + by).rem_euclid(len) as usize].0
    }
}

/// What the ROM check found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Check {
    Empty,
    /// A known edition, by name.
    Found(&'static str),
    /// Why the ROM cannot be played (an unknown build names its SHA-256
    /// and the issue link).
    Problem(String),
}

/// Checks a ROM path as typed or picked.
pub fn check_game(path: &str) -> Check {
    let path = path.trim();
    if path.is_empty() {
        return Check::Empty;
    }
    match edition::find(Path::new(path)) {
        Ok(found) => Check::Found(found.edition.name),
        Err(problem) => Check::Problem(problem),
    }
}

/// The window's state.
pub struct Window {
    pub form: Form,
    pub controls: Controls,
    /// The button whose keyboard key is being captured.
    pub capture: Option<SnesButton>,
    pub tab: Tab,
    /// The screen the window opened on, for the Steam Deck recommendation.
    pub screen: Option<(u32, u32)>,
    /// Started by the Linux helper.
    pub linux: bool,
    pub check: Check,
    /// The path text [`Window::check`] is for.
    checked: String,
    /// The path picker.
    pub pick: fn() -> Option<PathBuf>,
    pub hero: Option<TextureHandle>,
    /// A message that replaces the status line (a failed save).
    pub error: Option<String>,
    /// Play was pressed while ready: the caller saves and closes.
    pub play: bool,
    sized: bool,
}

impl Window {
    /// The window over a `launcher.cfg` text.
    pub fn new(text: &str, screen: Option<(u32, u32)>, controls: Controls, linux: bool) -> Window {
        let form = Form::from_text(text);
        let checked = form.get("game").to_owned();
        Window {
            check: check_game(&checked),
            checked,
            form,
            controls,
            capture: None,
            tab: Tab::Play,
            screen,
            linux,
            pick: || None,
            hero: None,
            error: None,
            play: false,
            sized: false,
        }
    }

    /// The screen size in physical pixels, once eframe knows it.
    fn measure_screen(&mut self, ctx: &egui::Context) {
        if self.sized {
            return;
        }
        self.sized = true;
        let native = ctx.native_pixels_per_point().unwrap_or(1.0);
        if let Some(monitor) = ctx.input(|i| i.viewport().monitor_size) {
            let w = (monitor.x * native).round() as u32;
            let h = (monitor.y * native).round() as u32;
            if w >= 640 && h >= 480 {
                self.screen = Some((w, h));
            }
        }
    }

    /// A known ROM is chosen and every value can be saved.
    pub fn ready(&self) -> bool {
        matches!(self.check, Check::Found(_)) && self.form.problems().is_empty()
    }

    /// Play (the plate, Enter or the pad's Start): only when ready; else
    /// back to the Play tab, where the reason is shown.
    pub fn press_play(&mut self) {
        if self.ready() {
            self.play = true;
        } else {
            self.tab = Tab::Play;
        }
    }

    fn refresh_check(&mut self) {
        if self.form.get("game") != self.checked {
            self.checked = self.form.get("game").to_owned();
            self.check = check_game(&self.checked);
        }
    }

    /// While a Change button waits for a key, the next key becomes that
    /// button's RetroArch key; Escape cancels.
    fn capture_key(&mut self, ctx: &egui::Context) {
        let Some(button) = self.capture else {
            return;
        };
        let pressed = ctx.input(|input| {
            input.events.iter().find_map(|event| match event {
                egui::Event::Key {
                    key, pressed: true, ..
                } => Some(*key),
                _ => None,
            })
        });
        if let Some(key) = pressed {
            if key == egui::Key::Escape {
                self.capture = None;
            } else if let Some(name) = retroarch_key(key) {
                self.controls.set_key(button, name.to_owned());
                self.capture = None;
            }
        }
    }
}

fn row(key: &str) -> &'static OptionSpec {
    spec::find(OPTIONS, key).unwrap_or_else(|| panic!("no option row {key}"))
}

/// The whole window, filling `ui`.
pub fn draw(ui: &mut egui::Ui, w: &mut Window) {
    w.capture_key(ui.ctx());
    w.refresh_check();
    let full = ui.max_rect();
    ui.painter().rect_filled(full, 0.0, BASE);
    let layout = Layout::new(full, PHOTO_W);
    draw_hero(ui, layout.photo, w.hero.as_ref());
    ui.scope_builder(egui::UiBuilder::new().max_rect(layout.body), |ui| {
        ui.add(
            egui::Label::new(
                RichText::new("Super Adventure Island")
                    .font(condensed(38.0))
                    .color(INK),
            )
            .wrap(),
        );
        status(ui, w);
        ui.add_space(6.0);
        widgets::tabs_bar(ui, &Tab::NAMES, &mut w.tab);
        ui.add_space(14.0);
        widgets::tab_body(ui, w.tab, |ui| match w.tab {
            Tab::Play => play_tab(ui, w),
            Tab::Picture => picture_tab(ui, w),
            Tab::Controls => controls_tab(ui, w),
            Tab::About => about_tab(ui),
        });
    });
    let hint = "Your ROM is never changed. Settings are saved when you press Play.";
    if widgets::footer(ui, layout.footer, hint, w.ready()).clicked() {
        w.press_play();
    }
}

fn draw_hero(ui: &egui::Ui, rect: Rect, hero: Option<&TextureHandle>) {
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, PANEL);
    // A sunset over the sea, drawn, not copied from the game.
    widgets::gradient(painter, rect, tint(ACCENT, 58), Fade::Down);
    if hero.is_none() {
        let sun = pos2(
            rect.left() + rect.width() * 0.62,
            rect.top() + rect.height() * 0.34,
        );
        painter.circle_filled(sun, 34.0, tint(ACCENT, 170));
        painter.text(
            pos2(rect.left() + 22.0, rect.top() + 24.0),
            Align2::LEFT_TOP,
            "SNES",
            body(13.0),
            MUTED,
        );
    }
    widgets::hero_image(painter, rect, hero, Crop::Center, BASE);
    if !HERO_CREDIT.is_empty() {
        painter.text(
            pos2(rect.left() + 20.0, rect.bottom() - 20.0),
            Align2::LEFT_BOTTOM,
            HERO_CREDIT,
            body(11.0),
            MUTED,
        );
    }
}

fn status(ui: &mut egui::Ui, w: &Window) {
    let problems = w.form.problems();
    let (text, color) = match (&w.error, &w.check) {
        (Some(error), _) => (error.clone(), ERROR),
        (None, Check::Found(_)) if !problems.is_empty() => (problems.join(" "), ERROR),
        (None, Check::Found(_)) => ("Ready. Press Play or Enter.".to_owned(), MUTED),
        (None, _) => (
            "First, show the launcher where the ROM is.".to_owned(),
            MUTED,
        ),
    };
    ui.label(RichText::new(text).font(body(14.0)).color(color));
}

fn play_tab(ui: &mut egui::Ui, w: &mut Window) {
    widgets::heading(ui, "Game");
    if spec_path_row(ui, row("game"), w.form.value_mut("game"), w.pick).changed() {
        w.error = None;
    }
    let (note, color) = match &w.check {
        Check::Empty if w.linux => (
            format!(
                "Choose the folder with {}, or the .sfc file itself.",
                edition::ROM_NAME
            ),
            MUTED,
        ),
        Check::Empty => (
            format!(
                "Choose the folder with {}, or the .sfc file itself.",
                edition::ROM_NAME
            ),
            MUTED,
        ),
        Check::Found(name) => (format!("Found {name}."), ACCENT),
        Check::Problem(problem) => (problem.clone(), ERROR),
    };
    ui.add(egui::Label::new(RichText::new(note).font(body(13.0)).color(color)).selectable(true));
    ui.add_space(12.0);

    widgets::heading(ui, row("look").label);
    // The cards are the `look` option's window control.
    mark_rendered(ui.ctx(), "look");
    ui.add_space(6.0);
    let recommended = Look::recommended(w.screen);
    let gap = 8.0;
    let width = ((ui.available_width() - 2.0 * gap) / 3.0).max(120.0);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        for look in Look::ALL {
            let note = look_note(look);
            let card = LookCard {
                title: look.label(),
                note: &note,
                selected: w.form.look() == Some(look),
                badge: (look == recommended).then_some("For this screen"),
                ..LookCard::default()
            };
            if widgets::look_card(ui, vec2(width, 96.0), &card).clicked() {
                w.form.choose_look(look, w.screen);
            }
        }
    });
    ui.add_space(6.0);
    widgets::note(ui, row("look").description);
}

/// A look card's note: its values in a few words.
fn look_note(look: Look) -> String {
    let p = look.picture(None);
    let aspect = match p.aspect.unwrap_or("") {
        "pixel" => "8:7",
        "4:3" => "4:3",
        "widescreen" => "Mode 7 16:9",
        _ => "core",
    };
    let scale = if p.integer_scale == Some(true) {
        "4×"
    } else {
        "smooth"
    };
    let shader = match p.shader.unwrap_or("off") {
        "crt" => "CRT",
        "sharp" => "sharp",
        "smooth" => "smooth",
        _ => "pixels",
    };
    let core = if p.core == Some("bsnes_hd") {
        "bsnes-hd"
    } else {
        "Snes9x"
    };
    format!("{aspect} · {scale} · {shader} · {core}")
}

/// Every game-affecting option that is not on the Play tab.
fn picture_tab(ui: &mut egui::Ui, w: &mut Window) {
    let mut changed = false;
    for row in OPTIONS
        .iter()
        .filter(|row| !row.launcher_only && !matches!(row.key, "game" | "look"))
    {
        changed |= spec_row(ui, row, w.form.value_mut(row.key)).changed();
        ui.add_space(10.0);
    }
    if changed {
        w.form.picture_edited(w.screen);
    }
}

fn about_tab(ui: &mut egui::Ui) {
    ui.style_mut().visuals.hyperlink_color = ACCENT;
    widgets::heading(ui, "Super Adventure Island Launcher");
    widgets::note(
        ui,
        "Runs your own ROM in RetroArch. It contains no game files and never changes your ROM.",
    );
    ui.add_space(10.0);
    widgets::heading(ui, "Built on");
    widgets::note(
        ui,
        "rea-launcher-sdk, RetroArch, Snes9x, bsnes-hd and the libretro slang shaders. \
         The DualSense autoconfig comes from RetroArch.",
    );
    ui.add_space(10.0);
    widgets::heading(ui, "Links");
    ui.hyperlink_to(
        "Project and issue tracker",
        "https://github.com/ghpo/rea-super-adventure-island",
    );
    ui.hyperlink_to(
        "rea-launcher-sdk",
        "https://github.com/akitaonrails/rea-launcher-sdk",
    );
    ui.hyperlink_to("RetroArch", "https://www.retroarch.com");
}

fn controls_tab(ui: &mut egui::Ui, w: &mut Window) {
    widgets::heading(ui, "Keyboard");
    widgets::note(
        ui,
        "Click Change and press the key. RetroArch key names: a-z, 0-9, up, down, left, \
         right, enter, space, backspace, tab, f1-f12. Escape cancels.",
    );
    ui.add_space(6.0);
    for button in SnesButton::ALL {
        ui.horizontal(|ui| {
            ui.add_sized(
                [110.0, 24.0],
                egui::Label::new(RichText::new(button.label()).font(body(14.0)).color(INK)),
            );
            let (text, color) = if w.capture == Some(button) {
                ("Press a key...".to_owned(), ACCENT)
            } else {
                (w.controls.key(button).to_owned(), INK)
            };
            ui.add_sized(
                [110.0, 24.0],
                egui::Label::new(RichText::new(text).font(body(14.0)).color(color)),
            );
            if widgets::outline_button(ui, "Change").clicked() {
                w.capture = Some(button);
            }
        });
    }
    ui.add_space(6.0);
    if widgets::outline_button(ui, "Reset controls").clicked() {
        w.controls.reset();
        w.capture = None;
    }
    ui.add_space(12.0);
    widgets::heading(ui, "Controller");
    widgets::note(
        ui,
        "Auto uses RetroArch's system autoconfig (the player's DualSense on Linux, XInput \
         on Windows). The presets set the standard pad's face and shoulder buttons; for a \
         fully custom pad, use RetroArch's Input settings.",
    );
    ui.add_space(4.0);
    ui.horizontal_wrapped(|ui| {
        for layout in PadLayout::ALL {
            if ui
                .selectable_label(w.controls.pad == layout, layout.label())
                .clicked()
            {
                w.controls.pad = layout;
            }
        }
    });
}

/// egui keys that make sense as RetroArch keyboard keys. Escape is the
/// capture cancel; modifiers and punctuation are left to `controls.cfg`.
fn retroarch_key(key: egui::Key) -> Option<&'static str> {
    use egui::Key;
    Some(match key {
        Key::A => "a",
        Key::B => "b",
        Key::C => "c",
        Key::D => "d",
        Key::E => "e",
        Key::F => "f",
        Key::G => "g",
        Key::H => "h",
        Key::I => "i",
        Key::J => "j",
        Key::K => "k",
        Key::L => "l",
        Key::M => "m",
        Key::N => "n",
        Key::O => "o",
        Key::P => "p",
        Key::Q => "q",
        Key::R => "r",
        Key::S => "s",
        Key::T => "t",
        Key::U => "u",
        Key::V => "v",
        Key::W => "w",
        Key::X => "x",
        Key::Y => "y",
        Key::Z => "z",
        Key::Num0 => "0",
        Key::Num1 => "1",
        Key::Num2 => "2",
        Key::Num3 => "3",
        Key::Num4 => "4",
        Key::Num5 => "5",
        Key::Num6 => "6",
        Key::Num7 => "7",
        Key::Num8 => "8",
        Key::Num9 => "9",
        Key::ArrowUp => "up",
        Key::ArrowDown => "down",
        Key::ArrowLeft => "left",
        Key::ArrowRight => "right",
        Key::Enter => "enter",
        Key::Space => "space",
        Key::Backspace => "backspace",
        Key::Tab => "tab",
        Key::F1 => "f1",
        Key::F2 => "f2",
        Key::F3 => "f3",
        Key::F4 => "f4",
        Key::F5 => "f5",
        Key::F6 => "f6",
        Key::F7 => "f7",
        Key::F8 => "f8",
        Key::F9 => "f9",
        Key::F10 => "f10",
        Key::F11 => "f11",
        Key::F12 => "f12",
        _ => return None,
    })
}

// ---------------------------------------------------------------- eframe

/// How the window ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Play was pressed; the settings are saved.
    Play,
    /// Closed without Play.
    Closed,
}

/// Why the settings window did not finish normally.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShowError {
    /// No window could be made (no display, no OpenGL): the caller may play
    /// with the saved settings.
    NoWindow(String),
    /// The window opened, then failed: the game must not start with stale
    /// settings.
    Failed(String),
}

fn finish(
    opened: bool,
    result: Result<(), String>,
    outcome: Outcome,
) -> Result<Outcome, ShowError> {
    match result {
        Ok(()) => Ok(outcome),
        Err(e) if opened => Err(ShowError::Failed(e)),
        Err(e) => Err(ShowError::NoWindow(e)),
    }
}

/// Shows the window over `saved` and saves to `path` on Play.
pub fn show(
    path: &Path,
    saved: Option<&str>,
    controls_path: &Path,
    controls: &Controls,
    linux: bool,
) -> Result<Outcome, ShowError> {
    use std::cell::Cell;
    use std::rc::Rc;

    use rea_launcher_ui::eframe;
    use rea_launcher_ui::pad::PadNav;
    use rea_launcher_ui::theme;
    use rea_launcher_ui::window::WindowFit;

    use crate::palette::{METRICS, PALETTE};

    let outcome = Rc::new(Cell::new(Outcome::Closed));
    let opened = Rc::new(Cell::new(false));
    let fit = WindowFit::new(WINDOW_SIZE, false, None, 1.0).with_native_fit(true);
    let options = fit.native_options(crate::WINDOW_TITLE);
    // The Windows VM has no OpenGL for eframe's default glow backend; the
    // wgpu backend uses D3D12 (including the WARP software adapter). The
    // Linux window stays on glow, which is verified.
    #[cfg(windows)]
    let options = {
        let mut options = options;
        options.renderer = eframe::Renderer::Wgpu;
        options
    };
    let text = saved.map_or_else(|| crate::config::first_run_text(None), str::to_owned);
    let path = path.to_path_buf();
    let controls_path = controls_path.to_path_buf();
    let controls = controls.clone();
    let shared = outcome.clone();
    let opened_flag = opened.clone();
    let result = eframe::run_native(
        crate::WINDOW_TITLE,
        options,
        Box::new(move |cc| {
            opened_flag.set(true);
            theme::install_with(&cc.egui_ctx, &PALETTE, &METRICS);
            let mut window = Window::new(&text, None, controls, linux);
            window.pick = pick_path;
            window.hero = HERO_JPEG.and_then(|jpeg| widgets::load_jpeg(&cc.egui_ctx, "hero", jpeg));
            Ok(Box::new(App {
                fit,
                window,
                path,
                controls_path,
                text: text.clone(),
                pad: PadNav::default(),
                pending: Vec::new(),
                outcome: shared,
                pads: Pads::new(),
            }))
        }),
    )
    .map_err(|e| e.to_string());
    finish(opened.get(), result, outcome.get())
}

struct App {
    fit: rea_launcher_ui::window::WindowFit,
    window: Window,
    path: PathBuf,
    controls_path: PathBuf,
    text: String,
    pad: rea_launcher_ui::pad::PadNav,
    pending: Vec<rea_launcher_ui::pad::Nav>,
    outcome: std::rc::Rc<std::cell::Cell<Outcome>>,
    pads: Pads,
}

impl App {
    fn save_and_close(&mut self, ctx: &egui::Context) {
        let text = self.window.form.save(&self.text);
        if let Err(error) = rea_launcher::fsx::write_atomic(&self.path, text) {
            self.window.error = Some(format!("Cannot save the settings: {error}"));
            return;
        }
        if let Err(error) = Controls::write(&self.controls_path, &self.window.controls) {
            self.window.error = Some(error);
            return;
        }
        self.outcome.set(Outcome::Play);
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

impl rea_launcher_ui::eframe::App for App {
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        let navs = self.pads.update(&mut self.pad);
        rea_launcher_ui::pad::inject(ctx, raw, &navs);
        self.pending.extend(navs);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut rea_launcher_ui::eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.window.measure_screen(&ctx);
        self.fit.first_frame(&ctx);
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) && self.window.capture.is_none() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        for nav in std::mem::take(&mut self.pending) {
            match nav {
                rea_launcher_ui::pad::Nav::Play if self.window.capture.is_none() => {
                    self.window.press_play()
                }
                rea_launcher_ui::pad::Nav::NextTab => self.window.tab = self.window.tab.step(1),
                rea_launcher_ui::pad::Nav::PreviousTab => {
                    self.window.tab = self.window.tab.step(-1)
                }
                _ => {}
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Enter))
            && !ctx.egui_wants_keyboard_input()
            && self.window.capture.is_none()
        {
            self.window.press_play();
        }
        draw(ui, &mut self.window);
        if std::mem::take(&mut self.window.play) {
            self.save_and_close(&ctx);
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(50));
    }
}

/// The pad as the SDK's XInput button mask. Linux reads every `/dev/input`
/// joystick; Windows reads XInput; other hosts have no pad.
struct Pads {
    #[cfg(target_os = "linux")]
    joysticks: rea_launcher_ui::pad::joystick::Joysticks,
}

impl Pads {
    fn new() -> Pads {
        Pads {
            #[cfg(target_os = "linux")]
            joysticks: rea_launcher_ui::pad::joystick::Joysticks::new(),
        }
    }

    fn buttons(&mut self) -> u16 {
        #[cfg(target_os = "linux")]
        {
            self.joysticks.buttons()
        }
        #[cfg(windows)]
        {
            rea_launcher_win32::xinput::buttons()
        }
        #[cfg(not(any(target_os = "linux", windows)))]
        {
            0
        }
    }

    fn update(&mut self, nav: &mut rea_launcher_ui::pad::PadNav) -> Vec<rea_launcher_ui::pad::Nav> {
        nav.update(self.buttons())
    }
}

/// The path picker: the SDK's dialog on Windows, zenity or kdialog on
/// Linux; it accepts a folder (the ROM file itself can be typed).
fn pick_path() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        rea_launcher_win32::dialog::pick_folder("Choose the folder with Super Adventure Island.sfc")
    }
    #[cfg(not(windows))]
    {
        use std::process::Command;
        let title = "Choose the folder with Super Adventure Island.sfc";
        for (program, args) in [
            (
                "zenity",
                vec!["--file-selection", "--directory", "--title", title],
            ),
            (
                "kdialog",
                vec!["--getexistingdirectory", ".", "--title", title],
            ),
        ] {
            if let Ok(out) = Command::new(program).args(&args).output()
                && out.status.success()
            {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !s.is_empty() {
                    return Some(PathBuf::from(s));
                }
            }
        }
        None
    }
}

/// A fatal error from the native build: printed to stderr and, when a
/// person may be watching on a desktop, shown in the SDK's message window.
#[cfg_attr(windows, allow(dead_code))]
pub fn fatal(text: &str, details: &str, says: rea_launcher_win32::report::Audience) {
    use rea_launcher_ui::report::{self, Message, Mode};
    let (text, link) = report_parts(text);
    let message = Message {
        title: crate::WINDOW_TITLE,
        text: &text,
        link,
        details: Some(details),
    };
    let mode = if says.command {
        Mode::Command {
            interactive: says.interactive,
        }
    } else {
        Mode::Play {
            interactive: says.interactive,
        }
    };
    report::fatal(
        &crate::palette::PALETTE,
        &crate::palette::METRICS,
        &message,
        mode,
    );
}

fn report_parts(text: &str) -> (String, Option<rea_launcher_ui::report::Link<'static>>) {
    use rea_launcher_ui::report::Link;
    if text.contains(edition::UNKNOWN_EDITION_ISSUE) {
        let text = text
            .replace(&format!(": {}", edition::UNKNOWN_EDITION_ISSUE), ".")
            .replace(edition::UNKNOWN_EDITION_ISSUE, "");
        return (
            text.trim_end().to_string(),
            Some(Link {
                label: "Report the ROM",
                url: edition::UNKNOWN_EDITION_ISSUE,
            }),
        );
    }
    (text.to_string(), None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{self, Settings};
    use crate::palette::{METRICS, PALETTE};
    use egui::Role;
    use rea_launcher_ui::spec::rendered_keys;
    use rea_launcher_ui::testing::{self, Harness, Queryable};

    fn open(text: &str) -> Harness<'static, Window> {
        testing::harness(
            &PALETTE,
            &METRICS,
            Window::new(text, Some((1920, 1080)), Controls::default(), false),
            draw,
        )
    }

    fn flags(args: &[&str]) -> Settings {
        let args = args.iter().map(|arg| (*arg).to_owned());
        config::parse_args(args).unwrap().0
    }

    #[test]
    fn the_window_has_a_control_for_every_option_and_no_other() {
        let mut ui = open("");
        ui.get_by_role_and_label(Role::Tab, "Picture").click();
        ui.run();
        spec::assert_parity(OPTIONS, &rendered_keys(&ui.ctx));
    }

    #[test]
    fn a_look_card_saves_what_its_flag_sets() {
        let mut ui = open("");
        ui.get_by_label("CRT").click();
        ui.run();
        let saved = ui.state().form.settings();
        assert_eq!(saved.look, Some(Look::Crt));
        let screen = ui.state().screen;
        let flag = flags(&["--look", "crt"]);
        assert_eq!(saved.picture(screen), flag.picture(screen));
        assert!(testing::checked(&ui.get_by_label("CRT")));
    }

    #[test]
    fn the_vsync_switch_saves_what_its_flag_sets() {
        let mut ui = open("vsync = on\n");
        ui.get_by_role_and_label(Role::Tab, "Picture").click();
        ui.run();
        testing::click_in_view(&mut ui, Role::Switch, "VSync");
        let saved = ui.state().form.settings();
        assert_eq!(saved.picture.vsync, Some(false));
        assert_eq!(
            saved.picture.vsync,
            flags(&["--vsync", "off"]).picture.vsync
        );
    }

    #[test]
    fn a_value_the_option_refuses_says_what_it_needs() {
        let mut ui = open("");
        *ui.state_mut().form.value_mut("shader") = "huge".into();
        ui.get_by_role_and_label(Role::Tab, "Picture").click();
        ui.run();
        assert!(ui.query_by_label_contains("Shader: needs").is_some());
    }

    #[test]
    fn play_waits_for_a_known_rom_and_an_unknown_one_is_refused() {
        let mut ui = open("");
        assert!(testing::disabled(
            &ui.get_by_role_and_label(Role::Button, "Play")
        ));

        let dir = std::env::temp_dir().join(format!("sai-ui-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(edition::ROM_NAME), b"not the game").unwrap();
        *ui.state_mut().form.value_mut("game") = dir.display().to_string();
        ui.run();
        let digest = rea_launcher::identify::hex(&rea_launcher::identify::sha256(b"not the game"));
        assert!(
            ui.query_by_label_contains(&digest).is_some(),
            "the SHA-256 is shown"
        );
        assert!(
            ui.query_by_label_contains(edition::UNKNOWN_EDITION_ISSUE)
                .is_some()
        );
        let play = ui.get_by_role_and_label(Role::Button, "Play");
        assert!(testing::disabled(&play));
        play.click();
        ui.run();
        assert!(!ui.state().play);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn tabs_step_and_wrap() {
        assert_eq!(Tab::Play.step(1), Tab::Picture);
        assert_eq!(Tab::Picture.step(1), Tab::Controls);
        assert_eq!(Tab::Controls.step(1), Tab::About);
        assert_eq!(Tab::About.step(1), Tab::Play);
        assert_eq!(Tab::Play.step(-1), Tab::About);
    }

    #[test]
    fn the_controls_tab_sets_a_pad_layout_and_shows_the_keyboard() {
        let mut ui = open("");
        ui.get_by_role_and_label(Role::Tab, "Controls").click();
        ui.run();
        assert!(ui.query_by_label_contains("Keyboard").is_some());
        ui.get_by_label("Xbox (A = A, B = B)").scroll_to_me();
        ui.run();
        ui.get_by_label("Xbox (A = A, B = B)").click();
        ui.run();
        assert_eq!(ui.state().controls.pad, PadLayout::Xbox);
    }

    #[test]
    fn a_change_button_captures_the_next_key() {
        let mut ui = open("");
        ui.get_by_role_and_label(Role::Tab, "Controls").click();
        ui.run();
        {
            let changes: Vec<_> = ui.get_all_by_label("Change").collect();
            assert_eq!(changes.len(), 12);
            changes[4].scroll_to_me();
        }
        ui.run();
        {
            let changes: Vec<_> = ui.get_all_by_label("Change").collect();
            changes[4].click();
        }
        ui.run();
        ui.key_press(egui::Key::K);
        ui.run();
        assert_eq!(ui.state().controls.key(SnesButton::A), "k");
        assert_eq!(ui.state().capture, None);
    }

    #[test]
    fn retroarch_key_maps_the_editable_keys() {
        assert_eq!(retroarch_key(egui::Key::Z), Some("z"));
        assert_eq!(retroarch_key(egui::Key::ArrowUp), Some("up"));
        assert_eq!(retroarch_key(egui::Key::F2), Some("f2"));
        assert_eq!(retroarch_key(egui::Key::Escape), None);
    }
}
