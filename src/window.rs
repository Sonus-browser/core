use std::num::NonZeroU32;
use std::rc::Rc;
use std::collections::HashMap;

use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent, MouseScrollDelta};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

/// Pixels are u32s in 0x00RRGGBB format.
const BACKGROUND: u32 = 0x00FF_FFFF; // white

struct PlacedGlyph {
    character: char,
    top: i32,
    left: usize,
}

/// Everything the app needs to remember between events.
/// The window only exists after the event loop starts, hence the Options.
struct App {
    text: String, // the page text from milestone 1; drawn in milestone 2b
    font: fontdue::Font,
    glyph_cache: HashMap<char, (fontdue::Metrics, Vec<u8>)>,
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    scroll: f32,
    glyphs: Vec<PlacedGlyph>,
    page_height: i32,
}

impl ApplicationHandler for App {
    /// Called once the OS is ready for us to create windows.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes().with_title("Sonus");
        let window = Rc::new(event_loop.create_window(attrs).unwrap());

        let context = Context::new(window.clone()).unwrap();
        let surface = Surface::new(&context, window.clone()).unwrap();

        self.layout(window.inner_size().width as usize);
        self.window = Some(window);
        self.surface = Some(surface);
    }

    /// Called for every event that happens to our window.
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            // Escape closes the window too
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => event_loop.exit(),

            WindowEvent::Resized(size) => {
                self.layout(size.width as usize);
                if let Some(window) = &self.window {
                    window.request_redraw();
                };
            },

            // The OS asks us to draw (on startup, after a resize, ...)
            WindowEvent::RedrawRequested => self.paint(),

            WindowEvent::MouseWheel { delta, .. } => {
                let dy = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y * 40.0,   // mouse wheel: counted in "lines"
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32, // trackpad: actual pixels
                };
                self.scroll -= dy;

                if let Some(window) = &self.window {
                    let max_scroll = self.page_height as f32 - window.inner_size().height as f32;
                    self.scroll = self.scroll.min(max_scroll);
                    self.scroll = self.scroll.max(0.0);
                    window.request_redraw();
                }
            },

            _ => {}
        }
    }
}

impl App {
    fn layout(&mut self, width: usize) {
        self.glyphs.clear();

        let size_px = 32.0;
        let mut baseline: i32 = 60;
        let mut cursor_x: f32 = 20.0;
        let space_width = self.font.metrics(' ', size_px).advance_width;
        let line_height = self.font.horizontal_line_metrics(size_px).unwrap().new_line_size;

        for word in self.text.split_whitespace() {
            let mut word_width = 0.0;
            for character in word.chars() {
                let (metrics, _) = self.glyph_cache.entry(character).or_insert_with(|| self.font.rasterize(character, size_px));
                word_width += metrics.advance_width;
            }

            let word_exceeds_width = cursor_x + word_width >= width as f32 - 20.0;
            if word_exceeds_width {
                cursor_x = 20.0;
                baseline += line_height.round() as i32;
            }
            
            for character in word.chars() {
                let (metrics, _) = self.glyph_cache.entry(character).or_insert_with(|| self.font.rasterize(character, size_px));
                let top = baseline - metrics.height as i32 - metrics.ymin;
                let left = (cursor_x + metrics.xmin as f32).round() as usize;

                self.glyphs.push(PlacedGlyph { character, top, left });
            
                cursor_x += metrics.advance_width; 
            }
            cursor_x += space_width;
        }
        self.page_height = baseline + 20;
    }

    fn paint(&mut self) {
        let (Some(window), Some(surface)) = (&self.window, &mut self.surface) else {
            return;
        };

        // The buffer must match the window's current size in pixels
        let size = window.inner_size();
        let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else {
            return; // minimized: 0x0, nothing to draw
        };
        surface.resize(w, h).unwrap();

        let mut buffer = surface.buffer_mut().unwrap();
        buffer.fill(BACKGROUND);

        let width = size.width as usize;
        let height = size.height as usize;
        let scroll = self.scroll as i32;

        for glyph in &self.glyphs {
            let (metrics, bitmap) = self.glyph_cache.get(&glyph.character).unwrap();
            let top = glyph.top - scroll;
            if top >= height as i32 {
                break;
            }
    
            // fully above the window: skip this letter
            if top + metrics.height as i32 <= 0 {
                continue;
             }

            for gy in 0..metrics.height {
                for gx in 0..metrics.width {
                    let coverage = bitmap[gy * metrics.width + gx] as u32;
                    let gray = 255 - coverage;
                    let color = (gray << 16) | (gray << 8) | gray;

                    let x = glyph.left + gx;
                    let y = top + gy as i32;
                    if y < 0 { continue; }
                    let y = y as usize;
                    if x < width && y < height {
                        buffer[y * width + x] = color;
                    }
                }
            }   
        }

        buffer.present().unwrap();
    }
}

    /// Open a window showing `text`. Blocks until the window is closed.
pub fn run(text: String) {
    let font_bytes = include_bytes!("../fonts/Play-Regular.ttf") as &[u8];
    let font = fontdue::Font::from_bytes(font_bytes, fontdue::FontSettings::default()).unwrap();    
    let glyph_cache = HashMap::new();
    let glyphs = Vec::new();
    let event_loop = EventLoop::new().unwrap();
    let mut app = App { text, font, glyph_cache, window: None, surface: None, scroll: 0.0, glyphs, page_height: 0 };
    event_loop.run_app(&mut app).unwrap();
}
