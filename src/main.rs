extern crate prototty;
extern crate prototty_unix;
use prototty::*;
use std::cmp;
mod hanoi;

use hanoi::*;

struct InteractState {
    hand_column: Stack,
    grabbed: Option<Stack>,
}

struct Game {
    game: GameState,
    interact: InteractState,
}

struct Rectangle {
    c: char,
    fore_colour: Rgb24,
    back_colour: Rgb24,
    top_left: Coord,
    bottom_right: Coord,
}
struct RectangleView{scale: u32}

impl View<Rectangle> for RectangleView {
    fn view<G: ViewGrid>(&mut self, rect: &Rectangle, offset: Coord, depth: i32, grid: &mut G) {
        for y in rect.top_left.y * self.scale as i32 .. rect.bottom_right.y * self.scale as i32 {
            for x in rect.top_left.x * self.scale as i32 .. rect.bottom_right.x * self.scale as i32 {
                if let Some(cell) = grid.get_mut(offset + Coord::new(x, y), depth) {
                    cell.set_character(rect.c);
                    cell.set_foreground_colour(rect.fore_colour);
                    cell.set_background_colour(rect.back_colour);
                }
            }
        }
    }
}

fn hanoi_rect(left: i32, right: i32, top: i32, bottom: i32, colour: Rgb24) -> Rectangle {
    Rectangle {
        c: ' ',
        fore_colour: colours::BLACK,
        back_colour: colour,
        top_left: Coord {x: left, y: top},
        bottom_right: Coord {x: right, y: bottom},
    }
}

pub struct Cylinder {
    middle: Coord,
    height: u32,
    radius: u32,
    colour: Rgb24,
}
pub struct CylinderView;
impl View<Cylinder> for CylinderView {
    fn view<G: ViewGrid>(&mut self, cylinder: &Cylinder, offset: Coord, depth: i32, grid: &mut G) {
        // render cylinder as a rectangle for now
        RectangleView{scale : 1}.view(
            &hanoi_rect(
                cylinder.middle.x - cylinder.radius as i32 + 1,
                cylinder.middle.x + cylinder.radius as i32,
                cylinder.middle.y - cylinder.height as i32,
                cylinder.middle.y + cylinder.height as i32 + 1,
                cylinder.colour
            ), offset, depth, grid);
    }
}

pub struct HanoiView { scale: u32 }
impl View<Game> for HanoiView {
    fn view<G: ViewGrid>(&mut self, game: &Game, offset: Coord, depth: i32, grid: &mut G) {
        // a column is 2 units, each piece is an additional 2 units and we need a 1 unit
        // border all the way around + 2 for the base
        let column_width = 2;
        let biggest_piece = (game.game.num_pieces() as i32 + 1) * 2 + column_width;
        let base_width = biggest_piece + 2;
        let reserve = base_width + 2;
        let hand_start = reserve * game.interact.hand_column as i32;
        RectangleView{scale: self.scale}.view(&hanoi_rect(hand_start, hand_start + reserve, 0, 1, colours::RED), offset, depth, grid);

        for col in 0..game.game.num_stacks() as i32 {
            let col_base = reserve * col;
            // render vertical column portion
            let upright_x = col_base + reserve / 2 - column_width / 2;
            let base_y = 3 + game.game.num_pieces() as i32 + 1;
            RectangleView{scale: self.scale}.view(&hanoi_rect(upright_x, upright_x + column_width, 3, base_y, colours::WHITE), offset, depth, grid);
            RectangleView{scale: self.scale}.view(&hanoi_rect(col_base + 1, col_base + 1 + base_width, base_y, base_y + 1, colours::WHITE), offset, depth, grid);
        }
        for piece_num in 0..game.game.num_pieces() {
            let piece = game.game.get_piece(piece_num);
            let mut col_base = reserve * piece.state.stack as i32;
            let mut base_y = 3 + game.game.num_pieces() as i32 - piece.state.height as i32;
            if let Some(stack) = game.interact.grabbed {
                // assume we did grab something
                if game.game.stack_top(stack).expect("Grabbed nothing?").num == piece_num {
                    base_y = 1;
                    col_base = reserve * game.interact.hand_column as i32;
                }
            }
            RectangleView{scale:self.scale}.view(&hanoi_rect(col_base + 2 + piece.num as i32, col_base + reserve - 2 - piece.num as i32, base_y, base_y + 1,
                    match piece.colour() { Colour::Black => colours::DARK_GREY, Colour::White => colours::YELLOW}
                ), offset, depth, grid);
        }
    }
}
impl ViewSize<Game> for HanoiView {
    fn size(&mut self, game: &Game) -> Size {
        Size::new(((game.game.num_pieces() + 1) * 2 + 7) * game.game.num_stacks() * self.scale, (game.game.num_pieces() + 5) * self.scale)
    }
}

fn make_scale<R: Renderer, D, V: ViewSize<D>>(context: &R, v: &mut V, data: &D) -> u32 {
    let term_size = context.size();
    let render_size = v.size(data);
    let scale_x = term_size.x() / render_size.x();
    let scale_y = term_size.y() / render_size.y();
    cmp::min(scale_x, scale_y)
}

fn main() {

    let mut context = prototty_unix::Context::new().unwrap();

    let mut game = Game {
        game: GameState::new(0, 3, 2).unwrap(),
        interact: InteractState { hand_column: 0, grabbed: None },
    };
    let scale = make_scale(&context, &mut HanoiView {scale : 1},&game);
    context.render(&mut HanoiView { scale: scale}, &game).unwrap();

    while let Ok(input) = context.wait_input() {
        let mut done = false;
        match input {
            Input::Left =>
                if game.interact.hand_column > 0 {
                    game.interact.hand_column = game.interact.hand_column - 1;
                },
            Input::Right =>
                if game.interact.hand_column + 1 < game.game.num_stacks() {
                    game.interact.hand_column = game.interact.hand_column + 1;
                },
            Input::Char(' ') => {
                    match game.interact.grabbed {
                        None =>
                            game.interact.grabbed = match game.game.stack_top(game.interact.hand_column) {
                                None => None,
                                Some(_) => Some(game.interact.hand_column),
                            },
                        Some(col) =>
                            game.interact.grabbed = match game.game.try_move(col, game.interact.hand_column) {
                                Ok(true) => None,
                                Ok(false) => Some(col),
                                _ => panic!("error"),
                            },
                    };
                    if game.game.complete() {
                        done = true;
                    }
                },
            _ => done = true,
        }
        let scale = make_scale(&context, &mut HanoiView {scale : 1},&game);
        context.render(&mut HanoiView { scale: scale}, &game).unwrap();
        if done { break; }
    }
}
