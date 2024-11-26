use std::fs;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use clap::Parser;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

// Import the rand crate's Rng trait
use rand::Rng;

/// A simple program to animate ASCII art in the terminal
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run the animation forever
    #[arg(short, long)]
    forever: bool,
}

fn main() -> crossterm::Result<()> {
    let args = Args::parse();

    // Read the ASCII art file
    let img_path = "/env/dot/bresilla/ascii"; // Update this path if necessary
    let img_content = match fs::read_to_string(img_path) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Error: Could not read the ASCII art file at '{}'", img_path);
            return Ok(());
        }
    };

    // Split the image into lines
    let img_lines: Vec<&str> = img_content.lines().collect();

    // Get image dimensions
    let img_height = img_lines.len();
    let img_width = img_lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);

    // Animation parameters
    let timeer = if args.forever { 0.1 } else { 0.05 }; // Adjusted timing
    let step = 3;
    let frames = img_height * step;

    // Colors
    let mut colors = vec![
        vec![
            Color::Grey,
            Color::Red,
            Color::Yellow,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
            Color::Green,
        ],
        vec![
            Color::Red,
            Color::Yellow,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
            Color::Grey,
            Color::Black,
        ],
    ];

    // Setup terminal
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;
    terminal::enable_raw_mode()?;

    // Handle Ctrl+C to exit gracefully
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    // Animation loop
    'outer: loop {
        let t_values: Vec<i32> = (-((frames as i32))..=(frames as i32))
            .step_by(step as usize)
            .collect();

        for t in t_values.iter() {
            if !running.load(Ordering::SeqCst) {
                break 'outer;
            }

            // Clear the screen
            queue!(stdout, terminal::Clear(ClearType::All))?;

            // Get terminal size
            let (cols, rows) = terminal::size()?;
            let cols = cols as usize;
            let rows = rows as usize;

            // Calculate offsets to center the image
            let start_y = if rows > img_height {
                (rows - img_height) / 2
            } else {
                0
            };
            let start_x = if cols > img_width {
                (cols - img_width) / 2
            } else {
                0
            };

            // For each line in the image
            for (y, line) in img_lines.iter().enumerate() {
                // Move to the correct position
                queue!(stdout, MoveTo(start_x as u16, (start_y + y) as u16))?;

                // For each character in the line
                for (x, ch) in line.chars().enumerate() {
                    let color = get_color(x, y, *t, &colors, img_height, img_width);
                    queue!(stdout, SetForegroundColor(color), Print(ch), ResetColor)?;
                }
            }

            stdout.flush()?;

            // Sleep for a bit
            thread::sleep(Duration::from_secs_f64(timeer));

            // Handle color rotation when t == 0
            if *t == 0 {
                let mut rng = rand::thread_rng();
                let idx = rng.gen_range(1..4); // Random integer between 1 and 3
                let temp = colors[0].remove(idx);
                colors[0].push(temp);

                if !args.forever {
                    break 'outer;
                }
            }

            // Handle key events
            while event::poll(Duration::from_millis(0))? {
                if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                    match code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => {
                            running.store(false, Ordering::SeqCst);
                            break 'outer;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Restore terminal
    execute!(stdout, Show, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}

fn get_color(
    x: usize,
    y: usize,
    t: i32,
    colors: &Vec<Vec<Color>>,
    img_height: usize,
    img_width: usize,
) -> Color {
    let max_img = std::cmp::max(img_height, img_width);
    let f = x as i32 - max_img as i32 + t.abs();
    let off = rand::thread_rng().gen_range(1..=15); // Random between 1 and 15

    for i in (0..=6).rev() {
        if f > y as i32 + (i as i32 * 3) + off {
            if t >= 0 {
                return colors[1][i];
            } else {
                return colors[0][i];
            }
        }
    }

    if t <= 0 {
        Color::Black
    } else {
        colors[0][colors[0].len() - 1]
    }
}
