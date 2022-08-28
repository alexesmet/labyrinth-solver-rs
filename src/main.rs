use std::convert::TryInto;

use sfml::{graphics::{Color, RectangleShape, Shape, Transformable}, system::Vector2f};
use imageproc;
use image;

const CELL_SIZE: u32 = 16;
const WALL_SIZE: u32 = 2;
const VIEW_SCALE: f32 = 8.0;
const FILENAME: &str = "labyrinth.png";


struct Cell {
    to_upper: bool,
    to_right: bool,
    to_down:  bool,
    to_left:  bool,
    status:   u8
}

impl Cell {
    fn exits(&self) -> u8 {
        let mut exits = 0;
        if self.to_upper { exits += 1 };
        if self.to_right { exits += 1 };
        if self.to_down  { exits += 1 };
        if self.to_left  { exits += 1 };
        return exits;
    }
}

#[derive(Clone)]
struct Walker {
    coords: (usize, usize),
    path: Vec<(usize, usize)>
}

fn main() {
    let image = sfml::graphics::Image::from_file(FILENAME)
        .expect("Couldn't open image");
    
    
    let lab_size = (
        (image.size().x - WALL_SIZE) / CELL_SIZE,
        (image.size().y - WALL_SIZE) / CELL_SIZE
    );

    let mut window = sfml::graphics::RenderWindow::new(
        (image.size().x / VIEW_SCALE as u32, image.size().y / VIEW_SCALE as u32),
        "Labyrinths solver",
        sfml::window::Style::CLOSE,
        &Default::default()
    );
    window.set_vertical_sync_enabled(false);
    let mut view = sfml::graphics::View::new(
        (image.size().x as f32 / 2.0, image.size().y as f32 / 2.0).into(),
        (image.size().x as f32 / 1.0, image.size().y as f32 / 1.0).into(),
    );
    sfml::graphics::RenderTarget::set_view(&mut window, &view);

    let texture = sfml::graphics::Texture::from_image(&image).unwrap();
    let sprite = sfml::graphics::Sprite::with_texture(&texture);

    let mut cells: Vec<Cell> = Vec::with_capacity((lab_size.0 * lab_size.1)
        .try_into()
        .expect("This labyrinth is too big for this computer."));

    let mut walkers: Vec<Walker> = vec![Walker {
        coords: (0, 0), path: vec![]
    }];
    
    for y in 0..lab_size.1 {
        for x in 0..lab_size.0 {
            let xc = x * CELL_SIZE + 1;
            let yc = y * CELL_SIZE + 1;
            let to_upper = image.pixel_at(xc+WALL_SIZE,yc          ).r > 0;
            let to_right = image.pixel_at(xc+CELL_SIZE,yc+WALL_SIZE).r > 0;
            let to_down  = image.pixel_at(xc+WALL_SIZE,yc+CELL_SIZE).r > 0;
            let to_left  = image.pixel_at(xc          ,yc+WALL_SIZE).r > 0;
            cells.push( Cell {
                to_upper, to_right, to_down, to_left, status: 0
            });
        }
    }
    cells[(lab_size.0 * lab_size.1 - 1) as usize].status = 2;

    let in_bounds = | c: (usize,usize) | c.0 < lab_size.0 as usize && c.1 < lab_size.1 as usize;

    let mut life_rects: Vec<RectangleShape> = vec![];

    let mut pause = true;

    loop {
        while let Some(event) = window.poll_event() {
            match event {
                sfml::window::Event::Closed => return,
                sfml::window::Event::KeyPressed { code: sfml::window::Key::Space, .. } => pause = !pause,
                _ => {}
            }
        }

        if !pause {
            let mut i = 0;
            let mut walkers_buffer: Vec<Walker> = Vec::with_capacity(4);
            while i < walkers.len() {
                let mut do_not_increment = false;
                // update walker
                let mut current = &mut walkers[i];
                let coords = current.coords.clone();
                current.path.push(coords.clone());
                // update cell
                let mut cell = &mut cells[coords.0 + lab_size.0 as usize * coords.1];
                if cell.status == 2 {
                    println!("WIN");
                }
                cell.status = 1;
                let cell = &cells[coords.0 + lab_size.0 as usize * coords.1];
                // write a smol function
                let is_vacant = | c: (usize, usize) |
                    in_bounds(c) && cells[c.0 + lab_size.0 as usize * c.1].status != 1;
                
                match cell.exits() {
                    // Dead end, this walker dies.
                    1 => { walkers.remove(i); do_not_increment = true;  },
                    // Walker should move forward
                    2 => {
                        let c_right = (coords.0 +1, coords.1   );
                        let c_down  = (coords.0   , coords.1 +1);
                        if coords.1 > 0 && cell.to_upper && is_vacant((coords.0   , coords.1 -1)) {
                            current.coords = (coords.0   , coords.1 -1);
                        } else if cell.to_right && is_vacant(c_right) {
                            current.coords = c_right;
                        } else if cell.to_down  && is_vacant(c_down)  {
                            current.coords = c_down;
                        } else if coords.0 > 0 && cell.to_left  && is_vacant((coords.0 -1, coords.1   ))  {
                            current.coords = (coords.0 -1, coords.1   );
                        } else {
                            // Path is blocked, this walker dies.
                            walkers.remove(i); do_not_increment = true;
                        }
                    },
                    // A cloning process should occur.
                    3 | 4 => {
                        let c_right = (coords.0 +1, coords.1   );
                        let c_down  = (coords.0   , coords.1 +1);
                        if coords.1 > 0 && cell.to_upper && is_vacant((coords.0   , coords.1 -1)) {
                            let mut cloned = current.clone();
                            cloned.coords = (coords.0   , coords.1 -1);
                            walkers_buffer.push(cloned);
                        }
                        if cell.to_right && is_vacant(c_right) {
                            let mut cloned = current.clone();
                            cloned.coords = c_right;
                            walkers_buffer.push(cloned);
                        }
                        if cell.to_down  && is_vacant(c_down)  {
                            let mut cloned = current.clone();
                            cloned.coords = c_down;
                            walkers_buffer.push(cloned);
                        } 
                        if coords.0 > 0 && cell.to_left  && is_vacant((coords.0 -1, coords.1   ))  {
                            let mut cloned = current.clone();
                            cloned.coords = (coords.0 -1, coords.1   );
                            walkers_buffer.push(cloned);
                        } 
                        // original dies, and that's sad :(
                        walkers.remove(i); do_not_increment = true; 

                    },
                    n => panic!("Recieved impossible number of exits: {}", n)
                }
                if !do_not_increment { i += 1; }
            }
            for moved in walkers_buffer.drain(..) {
                walkers.push(moved);
            }
            let mut i = 0;
            while i < life_rects.len() {
                let mut r = &mut life_rects[i];
                let life = r.fill_color().g;
                if life > 40 {
                    r.set_fill_color(Color::rgb(0, life -1, life -1));
                    i += 1;
                } else {
                    life_rects.remove(i);
                }
            }
        }

        use sfml::graphics::RenderTarget;

        // window.draw(&sprite);
        for r in life_rects.iter() {
            window.draw(r);
        }
        // draw walkers
        for w in walkers.iter() {
            let mut rect = RectangleShape::with_size( (12.0,12.0).into());
            rect.set_position((
                    w.coords.0 as f32 * CELL_SIZE as f32 + 3.0, 
                    w.coords.1 as f32 * CELL_SIZE as f32 + 3.0
            ));
            rect.set_fill_color(Color::rgb(255, 255, 255));
            window.draw(&rect);
            life_rects.push(rect);
        }
        window.display()


    }
}
