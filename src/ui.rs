

// KIND OF OUTDATED FILE , WILL MAKE A NEW ONE WHEN THE MAIN FILES ARE READY

use crossterm::style::{Color, Stylize};
use std::io::{self, Write};

pub struct ChiralUI {
    theme_color: Color,
    theme_label: &'static str,
    bar_width: usize,
    tick_counter: usize,
}

impl ChiralUI {
    pub fn new(is_levo: bool) -> Self {
        let theme_color = if is_levo {
            Color::Rgb { r: 168, g: 85, b: 247 }  // Levo-Purple
        } else {
            Color::Rgb { r: 249, g: 115, b: 22 }   // Dextro-Orange
        };

        let theme_label = if is_levo { "LEVO-PURPLE" } else { "DEXTRO-ORANGE" };

        Self {
            theme_color,
            theme_label,
            bar_width: 32,
            tick_counter: 0,
        }
    }

    pub fn draw_header(&self, version: &str) {
        println!(
            "\n🧬 Chiral Package Manager Engine v{} │ Chemistry: {}\n",
            version,
            self.theme_label.with(self.theme_color).bold()
        );
    }

    pub fn render_progress_frame(
        &mut self,
        current_step: usize,
        total_steps: usize,
        active_tasks: &[String],
        has_mutated: bool,
    ) {
        self.tick_counter = self.tick_counter.wrapping_add(1);

        let progress = if total_steps == 0 {
            1.0
        } else {
            current_step as f32 / total_steps as f32
        };

        let filled_chars = (progress * self.bar_width as f32).round() as usize;
        let heavy_load = !active_tasks.is_empty();

        let bracket_style = if heavy_load {
            let pulse = (self.tick_counter as f32 * 0.2).sin().abs();
            if has_mutated {
                Color::Red
            } else if pulse > 0.5 {
                self.theme_color
            } else {
                Color::DarkGrey
            }
        } else {
            Color::Grey
        };

        let status_msg = if has_mutated {
            "MUTATION EXCISED".red().bold().to_string()
        } else if heavy_load {
            "SYNTHESIZING    ".yellow().to_string()
        } else {
            "STABILIZED      ".dark_grey().to_string()
        };

        print!("\r\x1B[2K  {} {}", status_msg, "[".with(bracket_style));

        for x in 0..self.bar_width {
            if x < filled_chars {
                let phase = (x + self.tick_counter / 2) % 4;
                if has_mutated && x > 10 && x < 22 {
                    let mut_char = match phase {
                        0 | 2 => "╪",
                        1 | 3 => "✕",
                        _ => " ",
                    };
                    print!("{}", mut_char.red().bold());
                } else {
                    let helix_char = match phase {
                        0 => "⬢",
                        1 => "⤫",
                        2 => "⬡",
                        3 => "⤫",
                        _ => " ",
                    };
                    print!("{}", helix_char.with(self.theme_color));
                }
            } else if x == filled_chars && current_step != total_steps {
                let enzyme = ["⏵", "▹", "▸", "▹"][(self.tick_counter / 2) % 4];
                if has_mutated {
                    print!("{}", enzyme.red().bold());
                } else {
                    print!("{}", enzyme.with(self.theme_color));
                }
            } else {
                print!("{}", "·".dark_grey());
            }
        }

        let pct = (progress * 100.0).round() as usize;
        let delta_g = -(progress * 214.6);

        print!("] {:>3}% │ ΔG°: {:>6.1} kJ/mol", pct, delta_g);

        if !active_tasks.is_empty() {
            let tasks_joined = active_tasks.join(", ");
            let display_tasks = if tasks_joined.len() > 28 {
                format!("{}...", &tasks_joined[..25])
            } else {
                tasks_joined
            };
            print!(" │ Cores: [{}]", display_tasks.cyan());
        }

        io::stdout().flush().unwrap();
    }

    pub fn finish(&self) {
        println!("\n\n Verification complete. Structural isomer synthesis stabilized successfully.\n");
    }
}
