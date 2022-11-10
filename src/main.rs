extern crate sys_info;
extern crate colored;

use colored::Colorize;

fn main() {
    let mut lines = vec![Line::empty()];
    if let Ok(cpu_num) = sys_info::cpu_num() {
        let st = cpu_num.to_string();
        let mut st = st.chars();
        let st = match (st.next(), st.next()) {
            (None, _) => vec![" ".to_string(), " ".to_string()],
            (Some(s), None) => vec![s.to_string(), " ".to_string()],
            (Some(a), Some(b)) => vec![a.to_string(), b.to_string()],
        };
        lines.push(Line {
            colors_left: vec![Color {f: (100, 180, 0), b: None}],
            colors_right: vec![],
            color_overlay_mode: ColorOverlayMode::AlwaysLeft,
            progress: 0.0,
            short: Some(st),
            underline: false,
            long: Some(format!("{}", "logical cpu cores".green())),
        });
    }
    if let Ok(mem_info) = sys_info::mem_info() {
        let avail = mem_info.avail as f64 / mem_info.total as f64;
        let unit = "gb".truecolor(90, 90, 130);
        lines.push(Line {
            colors_left: vec![Color {f: (150, 0, 100), b: None}],
            colors_right: vec![Color {f: (0, 100, 150), b: None}],
            color_overlay_mode: ColorOverlayMode::ChooseFromPosFade(0.1),
            progress: (2.0 - 2.0 * avail).min(1.0),
            short: Some("ram/".chars().map(|c| c.to_string()).collect()),
            underline: true,
            long: Some(format!("total: {}{} ({} used)",
                format!("{:.1}", mem_info.total as f64 / 1024.0 / 1024.0).magenta(), unit,
                format!("{:.1}%", 100.0 - avail * 100.0).red()
            )),
        });
        lines.push(Line {
            colors_left: vec![Color {f: (150, 0, 100), b: None}],
            colors_right: vec![Color {f: (0, 100, 150), b: None}],
            color_overlay_mode: ColorOverlayMode::ChooseFromPosFade(0.1),
            progress: 1.0 - (avail * 2.0).min(1.0),
            short: Some("memory".chars().map(|c| c.to_string()).collect()),
            underline: true,
            long: Some(format!("free: {}{} ({})",
                format!("{:.1}", mem_info.avail as f64 / 1024.0 / 1024.0).blue(), unit,
                format!("{:.1}%", avail * 100.0).blue()
            )),
        });
    }
    if let (Ok(os_type), Ok(os_release), Ok(hostname)) = (sys_info::os_type(), sys_info::os_release(), sys_info::hostname()) {
        lines.push(Line {
            colors_left: vec![Color {f: (90, 90, 200), b: None}],
            colors_right: vec![],
            color_overlay_mode: ColorOverlayMode::AlwaysLeft,
            progress: 0.0,
            short: Some(os_type.chars().map(|c| c.to_string()).collect()),
            underline: false,
            long: Some(format!("{} {} {}", hostname.red(), "@".truecolor(128, 128, 128), os_release.cyan())),
        });
    }
    lines.push(Line::empty());
    let h = lines.len();
    let mut space = h;
    for line_index in 1..h {
        let line = &lines[line_index-1];
        let width = 2 * (h - space);
        space -= 1;
        for _ in 0..space { print!(" "); }
        print!("╱");
        let short = &line.short;
        let short = match short { Some(short) => short.iter().collect(), None => Vec::new() };
        let minx = if short.len() >= width { 0 } else { (1 + width - short.len()) / 2 };
        for x in 0..width {
            let xf = x as f64 / (width - 1) as f64;
            let text = line.get_color(xf).apply(if x >= minx { match short.get(x - minx) { Some(s) => s.as_str(), None => " " } } else { " " });
            let text = if line.underline { text.underline() } else { text };
            print!("{}", text);
        }
        print!("╲");
        if let Some(long) = &line.long {
            println!(" {}", long);
        } else { println!(); }
    }
    // print!(" ");
    print!(" "); for _ in 1..h { print!("{}", "⎺⎺"); }
    println!();
}

struct Line {
    colors_left: Vec<Color>,
    colors_right: Vec<Color>,
    color_overlay_mode: ColorOverlayMode,
    progress: f64,
    short: Option<Vec<String>>,
    underline: bool,
    long: Option<String>,
}
impl Line {
    pub fn empty() -> Self { Self {
        colors_left: vec![Color { f: (0, 100, 150), b: None }],
        colors_right: vec![],
        color_overlay_mode: ColorOverlayMode::AlwaysLeft,
        progress: 0.5,
        short: None,
        underline: false,
        long: None,
    } }
    pub fn get_color(&self, split: f64) -> Color {
        match self.color_overlay_mode {
            ColorOverlayMode::AlwaysLeft => self.colors_left[(split * (self.colors_left.len() - 1) as f64) as usize],
            ColorOverlayMode::AlwaysRight => self.colors_right[(split * (self.colors_right.len() - 1) as f64) as usize],
            ColorOverlayMode::ChooseFromPos => if split < self.progress {
                self.colors_left[(split * (self.colors_left.len() - 1) as f64) as usize]
            } else {
                self.colors_right[(split * (self.colors_right.len() - 1) as f64) as usize]
            },
            ColorOverlayMode::ChooseFromPosFade(d) => {
                // println!("a: {:.2} | b: {:.2}", self.progress, split);
                let h = d / 2.0;
                let mid = self.progress * (1.0 + d) - h;
                let diff = ((split - mid) + h) / d;
                // println!("mid: {:.2} | diff: {:.2}", mid, diff);
                if diff <= 0.0 {
                    self.colors_left[(split * (self.colors_left.len() - 1) as f64) as usize]
                } else if diff >= 1.0 {
                    self.colors_right[(split * (self.colors_right.len() - 1) as f64) as usize]
                } else {
                    let l = self.colors_left[(split * (self.colors_left.len() - 1) as f64) as usize];
                    let r = self.colors_right[(split * (self.colors_right.len() - 1) as f64) as usize];
                    Color::fade(l, r, diff)
                }
            },
            ColorOverlayMode::Stretch => if split <= self.progress {
                self.colors_left[((split / self.progress) * (self.colors_left.len() - 1) as f64) as usize]
            } else {
                self.colors_right[(((split - self.progress) / (1.0 - self.progress)) * (self.colors_right.len() - 1) as f64) as usize]
            },
            ColorOverlayMode::Fade => {
                let c1 = self.colors_left[(split * (self.colors_left.len() - 1) as f64) as usize];
                let c2 = self.colors_right[(split * (self.colors_right.len() - 1) as f64) as usize];
                Color::fade(c1, c2, split)
            },
        }
    }
}

#[allow(dead_code)]
enum ColorOverlayMode {
    AlwaysLeft,
    AlwaysRight,
    /// For chars on the left, choose from colors_left, for chars on the right, choose from colors_right
    ChooseFromPos,
    /// For chars on the left, choose from colors_left, for chars on the right, choose from colors_right. In a [float] wide area around where things are changing, fade the colors
    ChooseFromPosFade(f64),
    /// Stretch colors_left on the segment from 0 to the split, and stretch colors_right on the ramaining part
    Stretch,
    /// The split isn't a split at a certain position but rather controls how much of colors_right (and therefor how little of colors_left) is shown
    Fade,
}

#[derive(Clone, Copy)]
struct Color {
    pub f: (u8, u8, u8), 
    pub b: Option<(u8, u8, u8)>,
}
impl Color {
    pub fn apply(&self, s: &str) -> colored::ColoredString {
        if let Some(bg) = &self.b {
            s.truecolor(self.f.0, self.f.1, self.f.2)
                .on_truecolor(bg.0, bg.1, bg.2)
        } else {
            s.truecolor(self.f.0, self.f.1, self.f.2)
        }
    }
    pub fn fade(c1: Color, c2: Color, f: f64) -> Color {
        let g = 1.0 - f;
        Color {
            f: (
                (c1.f.0 as f64 * g + c2.f.0 as f64 * f) as u8,
                (c1.f.1 as f64 * g + c2.f.1 as f64 * f) as u8,
                (c1.f.2 as f64 * g + c2.f.2 as f64 * f) as u8,
            ),
            b: match (c1.b, c2.b) {
                (None, None) => None,
                (Some(c1), Some(c2)) => Some((
                    (c1.0 as f64 * g + c2.0 as f64 * f) as u8,
                    (c1.1 as f64 * g + c2.1 as f64 * f) as u8,
                    (c1.2 as f64 * g + c2.2 as f64 * f) as u8,
                )),
                _ => panic!("Cannot fade two colors if exactly one of them has a background!"),
            },
        }
    }
}
