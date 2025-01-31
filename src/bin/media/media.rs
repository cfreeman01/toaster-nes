use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::EventPump;
use sdl2::Sdl;
use sdl2::VideoSubsystem;

pub struct Media {
    context: Sdl,
    video_subsystem: VideoSubsystem,
    canvas: Canvas<Window>,
    event_pump: EventPump,
    native_width: u32,
    native_height: u32,
}

pub enum MediaEvent {
    Quit,
}

impl Media {
    pub fn init(title: &str, native_width: u32, native_height: u32, scale: u32) -> Self {
        let context = sdl2::init().unwrap();
        let video_subsystem = context.video().unwrap();

        let mut canvas = video_subsystem
            .window(title, native_width * scale, native_height * scale)
            .position_centered()
            .resizable()
            .build()
            .unwrap()
            .into_canvas()
            .build()
            .unwrap();

        let mut event_pump = context.event_pump().unwrap();

        canvas.set_logical_size(native_width, native_height);

        Self {
            context,
            video_subsystem,
            canvas,
            event_pump,
            native_width,
            native_height,
        }
    }

    pub fn poll_event(&mut self) -> Option<MediaEvent> {
        let mut media_event: Option<MediaEvent> = None;

        match self.event_pump.poll_event() {
            Some(Event::Quit { .. }) => media_event = Some(MediaEvent::Quit),
            _ => (),
        };

        media_event
    }

    pub fn render(&mut self, frame: &[u8]) {
        for x in 0..self.native_width {
            for y in 0..self.native_height {
                let idx = ((self.native_width * 3 * y) + (3 * x)) as usize;

                self.canvas.set_draw_color(Color {
                    r: frame[idx],
                    g: frame[idx + 1],
                    b: frame[idx + 2],
                    a: 0xff,
                });

                self.canvas.draw_point(Point::new(x as i32, y as i32));
            }
        }

        self.canvas.present();
    }
}
