use rand::Rng;
use std::io::{stdout, Write};
use std::time::{Duration, Instant};

use crossterm::{
    cursor::{MoveTo, RestorePosition, SavePosition},
    event::{self, Event, KeyCode},
    execute,
    style::{Color, SetBackgroundColor},
    terminal::{self, enable_raw_mode, Clear, ClearType},
    Result,
};

#[derive(Clone)]
struct Shape {
    v: Vec<Vec<i32>>,
}

#[derive(Clone)]
struct ShapeInstace {
    s: Shape,
    // x, y is the position of the shape of the top left corner
    x: usize,
    y: usize,
}

struct Board {
    // board width and height
    width: usize,
    height: usize,
    // current active shape instance
    current_active_block: Option<ShapeInstace>,
    // the freeze grid, 1 means the cell is occupied
    grid: Vec<Vec<i32>>,
}

impl Board {
    fn new(width: usize, height: usize) -> Board {
        Board {
            width,
            height,
            current_active_block: None,
            grid: vec![vec![0; width]; height],
        }
    }

    // define all the shapes, L, J, T, O, S, Z, I
    fn l() -> Shape {
        Shape {
            v: vec![vec![1, 0], vec![1, 0], vec![1, 1]],
        }
    }

    fn j() -> Shape {
        Shape {
            v: vec![vec![0, 1], vec![0, 1], vec![1, 1]],
        }
    }

    fn o() -> Shape {
        Shape {
            v: vec![vec![1, 1], vec![1, 1]],
        }
    }

    fn s() -> Shape {
        Shape {
            v: vec![vec![0, 1, 1], vec![1, 1, 0]],
        }
    }

    fn z() -> Shape {
        Shape {
            v: vec![vec![1, 1, 0], vec![0, 1, 1]],
        }
    }

    fn i() -> Shape {
        Shape {
            v: vec![vec![1], vec![1], vec![1], vec![1]],
        }
    }

    fn random_shape() -> Shape {
        let shapes = vec![
            Self::l(),
            Self::j(),
            Self::o(),
            Self::s(),
            Self::z(),
            Self::i(),
        ];
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..shapes.len());
        return shapes[index].clone();
    }

    fn rotate_shape(shape: Shape) -> Shape {
        // Step 1: Transpose the matrix
        let mut transposed = vec![vec![0; shape.v.len()]; shape.v[0].len()];
        for i in 0..shape.v.len() {
            for j in 0..shape.v[i].len() {
                transposed[j][i] = shape.v[i][j];
            }
        }

        // Step 2: Reverse each row
        for row in transposed.iter_mut() {
            row.reverse();
        }

        Shape { v: transposed }
    }

    fn rotate_current_shape(&mut self) {
        if let Some(ab) = self.current_active_block.clone() {
            let rotated_shape = Board::rotate_shape(ab.s.clone());
            let rotated_instance = ShapeInstace {
                s: rotated_shape,
                x: ab.x,
                y: ab.y,
            };
            // if collision, then do nothing; otherwise set current active block as the roated one
            if self.check_collision(&rotated_instance) {
                return;
            } else {
                self.current_active_block = Some(rotated_instance);
            }
        }
    }

    fn get_current_active_shape(&mut self) -> ShapeInstace {
        // if active block is None, get a new random shape
        if self.current_active_block.is_none() {
            let shape = Board::random_shape();
            let si = ShapeInstace {
                s: shape.clone(),
                x: self.width / 2,
                y: 0,
            };
            self.current_active_block = Some(si);
        }
        return self
            .current_active_block
            .clone()
            .expect("Active block is None");
    }

    fn move_current_shape_down(&mut self) {
        if let Some(ab) = self.current_active_block.clone() {
            let ab_new_location = ShapeInstace {
                s: ab.s.clone(),
                x: ab.x,
                y: ab.y + 1,
            };
            // if collision, then add the shape to the grid and get a new shape; otherwise move down
            if self.check_collision(&ab_new_location) {
                self.add_shape(&ab);
                self.current_active_block = None;
            } else {
                self.current_active_block = Some(ab_new_location);
            }
        }
    }

    fn move_current_shape_left(&mut self) {
        if let Some(ab) = self.current_active_block.clone() {
            // if it is left most, then do nothing
            if ab.x == 0 {
                return;
            }

            // move to left, if there is collidion, then do nothing; otherwise move to left
            let ab_new_location = ShapeInstace {
                s: ab.s.clone(),
                x: ab.x - 1,
                y: ab.y,
            };
            if self.check_collision(&ab_new_location) {
                return;
            } else {
                self.current_active_block = Some(ab_new_location);
            }
        }
    }

    fn move_current_shape_right(&mut self) {
        if let Some(ab) = self.current_active_block.clone() {
            // if it is right most, then do nothing
            if ab.x + ab.s.v[0].len() == self.width {
                return;
            }

            // move to right, if there is collidion, then do nothing; otherwise move to right
            let ab_new_location = ShapeInstace {
                s: ab.s.clone(),
                x: ab.x + 1,
                y: ab.y,
            };
            if self.check_collision(&ab_new_location) {
                return;
            } else {
                self.current_active_block = Some(ab_new_location);
            }
        }
    }

    // add the shape instance to the grid of the board
    fn add_shape(&mut self, si: &ShapeInstace) {
        for j in 0..si.s.v.len() {
            for i in 0..si.s.v[j].len() {
                self.grid[(si.y + j) as usize][(si.x + i) as usize] = si.s.v[j][i];
            }
        }
    }

    fn check_collision(&self, si: &ShapeInstace) -> bool {
        // check if the shapeInstance is out of the board
        if si.x + si.s.v[0].len() > self.width as usize
            || si.y + si.s.v.len() > self.height as usize
        {
            return true;
        }

        // if the grid has 1 at the position of the shapeInstance, then it is a collision
        for j in 0..si.s.v.len() {
            for i in 0..si.s.v[j].len() {
                if self.grid[(si.y + j) as usize][(si.x + i) as usize] == 1 {
                    return true;
                }
            }
        }
        return false;
    }

    fn draw(&mut self) {
        // clear the screen
        execute!(stdout(), Clear(ClearType::All)).unwrap();

        // draw backgroud of the board with white color
        for j in 0..self.height {
            for i in 0..self.width {
                draw_block(i as u16, j as u16, Color::White).unwrap();
            }
        }

        // draw active shape
        let shape = self.get_current_active_shape();
        draw_shape(&shape.s, shape.x, shape.y, Color::Red).unwrap();

        // draw the grid
        for j in 0..self.height {
            for i in 0..self.width {
                if self.grid[j][i] == 1 {
                    draw_block(i as u16, j as u16, Color::Blue).unwrap();
                }
            }
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn get_grid(&self) -> Vec<Vec<i32>> {
        return self.grid.clone();
    }
}

/* drawing functions */
fn draw_block(x: u16, y: u16, color: Color) -> crossterm::Result<()> {
    let mut stdout = stdout();

    // Save the current position of the cursor
    execute!(stdout, SavePosition)?;

    // Set the background color for the block
    execute!(stdout, SetBackgroundColor(color))?;

    // Draw the block as 2x2 colored spaces
    // Todo: use 2x2 block looks much square but need do some math to calculate the position
    for i in 0..1 {
        execute!(stdout, MoveTo(x, y + i))?;
        stdout.write_all(b" ")?; // Each space character will be colored, creating a square
    }

    // Restore the cursor position and reset the background color
    execute!(stdout, RestorePosition, SetBackgroundColor(Color::Reset))?;

    stdout.flush()?;
    Ok(())
}

fn draw_shape(shape: &Shape, x: usize, y: usize, color: Color) -> crossterm::Result<()> {
    for j in 0..shape.v.len() {
        for i in 0..shape.v[j].len() {
            if shape.v[j][i] == 1 {
                draw_block(x as u16 + i as u16, y as u16 + j as u16, color)?;
            }
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    enable_raw_mode()?;

    let mut board = Board::new(20, 20);
    let mut last_draw_time = Instant::now();

    loop {
        // Wait for a key press
        if event::poll(std::time::Duration::from_millis(100))? {
            let evt = event::read()?;
            match evt {
                Event::Key(key_event) => {
                    match key_event.code {
                        KeyCode::Esc => break, // Exit on ESC
                        KeyCode::Char('c')
                            if key_event.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            break
                        } // Exit on Ctrl+C
                        KeyCode::Char(' ') => {
                            // rotate on space key
                            board.rotate_current_shape();
                        }
                        KeyCode::Left => {
                            board.move_current_shape_left();
                        }
                        KeyCode::Right => board.move_current_shape_right(),
                        KeyCode::Down => {
                            // TODO: Move shape down - double speed
                        }
                        _ => {} // Ignore other keys
                    }
                }
                _ => {} // Ignore other events
            }
        }

        // update the screen perodicly
        if last_draw_time.elapsed() >= Duration::from_millis(500) {
            board.move_current_shape_down();
            board.draw();

            last_draw_time = Instant::now();
        }
    }

    terminal::disable_raw_mode()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_shape() {
        let mut board = Board::new(10, 20);
        let shape = Board::l();
        let si = ShapeInstace {
            s: shape,
            x: 0,
            y: 0,
        };
        board.add_shape(&si);
        let grid = board.get_grid();
        assert_eq!(grid[0][0], 1);
        assert_eq!(grid[1][0], 1);
        assert_eq!(grid[2][0], 1);
        assert_eq!(grid[2][1], 1);
    }

    // test check collision
    #[test]
    fn test_check_collision() {
        let mut board = Board::new(10, 20);
        let shape = Board::l();
        let si = ShapeInstace {
            s: shape.clone(),
            x: 0,
            y: 0,
        };
        assert_eq!(board.check_collision(&si), false);

        // move instance to the bottom and add it
        let si2 = ShapeInstace {
            s: shape.clone(),
            x: 0,
            y: 20 - shape.v.len(),
        };
        board.add_shape(&si2);

        // check the collision with same instance shape
        assert_eq!(board.check_collision(&si2), true);
    }
}
