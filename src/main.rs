use bracket_lib::prelude::*;

// constants to better manage aspects of the game - note constants NEED types
const SCREEN_WIDTH: i32 = 40;
const SCREEN_HEIGHT: i32 = 25;
const FRAME_DURATION: f32 = 75.0;

// additional constant for the Dragon sprite
const DRAGON_FRAMES: [u16; 6] = [64, 1, 2, 3, 2, 1];

struct Player {
    x: i32, // x position = world-space position of the player & player will always render from the left
    y: f32, // y position = vertical position in the world
    velocity: f32, // Represents upward momentum - f32 allows for fractions which provides a smoother play experience
    frame: usize,  // usize provided to index arrays
}

impl Player {
    fn new(x: i32, y: i32) -> Self {
        Player {
            x,
            y: y as f32,
            velocity: 0.0, // f32 needs a .0 to bind correctly to a 0 value
            frame: 0,
        }
    }

    fn render(&mut self, ctx: &mut BTerm) {
        // the render() function will render the player as a YELLOW @ symbol on the left of the screen
        ctx.set_active_console(1);
        ctx.cls();
        ctx.set_fancy(
            // set() is a function that sets a single character on the screen
            PointF::new(0.0, self.y),
            1,
            Degrees::new(0.0),
            PointF::new(2.0, 2.0),
            WHITE,
            NAVY,
            DRAGON_FRAMES[self.frame], // copies the Unicode symbol from the code to matching Codepage437 char number
        );
        ctx.set_active_console(0);
    }

    // adding gravity to the player object
    fn gravity_and_move(&mut self) {
        // checking if the player character has reached its terminal velocity
        if self.velocity < 2.0 {
            self.velocity += 0.1;
        }

        // Add the velocity to the player's current 'y' position, but need to convert as you cannot add f32 and i32
        self.y += self.velocity;
        if self.y < 0.0 {
            self.y = 0.0;
        }
        // increment 'x' to know how far it has progressed through the level - incrementing tracks this
        self.x += 1;
        self.frame += 1;
        self.frame = self.frame % 6; // % is modulus - remainder
    }

    // add the ability to move by 'flapping' wings
    fn flap(&mut self) {
        // Flap sets the player character's velocity to -2.0 :- Negative = Upward as top is 0,0
        self.velocity = -1.0
    }
}

// Game modes are best represented as an enum
enum GameMode {
    Menu,
    Playing,
    End,
}

// state represents a snapshot of the current game
struct State {
    // player and frame_time added to the state struct
    player: Player,
    frame_time: f32,
    // State now tracks the current obstacle in play
    obstacle: Obstacle,
    mode: GameMode,
    // State now tracks the player's score based on how many obstacles hit
    score: i32,
}

// Creating a constructor to initialize the State struct
impl State {
    fn new() -> Self {
        State {
            // now the player construct exists you need to add it to the State constructor
            player: Player::new(5, 25), // Player positioned slightly right of the left side of screen
            frame_time: 0.0,            // frame_time initialized to 0 at the start
            obstacle: Obstacle::new(SCREEN_WIDTH, 0),
            mode: GameMode::Menu,
            score: 0,
        }
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        // print_centered is an extended version of print that centers text on a line given a y pos
        ctx.print_color_centered(5, YELLOW, BLACK, "Welcome to Flappy Dragon");
        // introduces the need to receive user input and act upon it - best representation of this is an Option type
        ctx.print_color_centered(8, CYAN, BLACK, "(P) Play Game");
        ctx.print_color_centered(9, CYAN, BLACK, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),    // P to Play Game actually restarts - it doesn't call play()
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {} // => {} instructs Rust to ignore any options not listed
            }
        }
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY); // sets the background of the context/play space to NAVY BLUE

        // slows down the game to a more manageable rate
        self.frame_time += ctx.frame_time_ms; // frame_time_ms contains the time elapsed since the last time tick was called

        // if frame_time exceeds FRAME_DURATION constant it is time to run the physics simulation and reset the frame to 0
        if self.frame_time > FRAME_DURATION {
            // reset time to 0
            self.frame_time = 0.0;
            // physics simulation
            self.player.gravity_and_move();
        }

        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }

        self.player.render(ctx);
        ctx.print(0, 0, "Press SPACE to flap.");
        // displays the player's current score underneath the instructions - does not send to stdout but returns a String
        ctx.print(0, 1, &format!("Score: {}", self.score));

        self.obstacle.render(ctx, self.player.x);
        if self.player.x > self.obstacle.x {
            self.score += 1;
            self.obstacle = Obstacle::new(self.player.x + SCREEN_WIDTH, self.score);
        }

        // self.player.y needs to be casted to i32
        if self.player.y as i32 > SCREEN_HEIGHT || self.obstacle.hit_obstacle(&self.player) {
            self.mode = GameMode::End;
        }
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_color_centered(5, RED, BLACK, "You are dead!");
        ctx.print_centered(6, &format!("You earned {} points", self.score));
        ctx.print_color_centered(8, CYAN, BLACK, "(P) Play Again");
        ctx.print_color_centered(9, CYAN, BLACK, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    // main menu function is complex - needs to display the menu & respond to user input + change mode to Playing & reset all game states
    fn restart(&mut self) {
        // to correctly model a game restarting the player position needs to be reset and the frame_time reset to 0
        self.player = Player::new(5, SCREEN_WIDTH / 2);
        self.frame_time = 0.0;
        self.obstacle = Obstacle::new(SCREEN_WIDTH, 0);
        self.mode = GameMode::Playing;
        self.score = 0;
    }
}

struct Obstacle {
    x: i32,     // defines the obstacle's position in the world-space
    gap_y: i32, // defines the centre of the gap through which the dragon passes
    size: i32,  // defines the length of the gap in the obstacle
}

impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        // bracket-lib uses the xorshift algorithm to generate a pseudo-random number
        let mut random = RandomNumberGenerator::new();
        Obstacle {
            x,
            gap_y: random.range(5, 20), // obstacles will have a y value between 10 & 39
            // gap's size is the maximum of (20 minus the player score, or 2)
            size: i32::max(2, 10 - score), // ensures that the gaps decrease but never less than 2
        }
    }

    fn render(&mut self, ctx: &mut BTerm, player_x: i32) {
        // rendering the ground
        for x in 0..SCREEN_WIDTH {
            ctx.set(x, SCREEN_HEIGHT - 1, WHITE, WHITE, to_cp437('#'));
        }

        let screen_x = self.x - player_x;
        let half_size = self.size / 2;

        // Draw the top half of the obstacle
        for y in 0..self.gap_y - half_size {
            ctx.set(screen_x, y, WHITE, NAVY, 179);
        }

        // Draw the bottom half of the obstacle with room for the ground!
        for y in self.gap_y + half_size..SCREEN_HEIGHT - 1 {
            ctx.set(screen_x, y, WHITE, NAVY, 179);
        }
    }

    fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size / 2;
        // if the player's X coordinate matches the obstacle's there might be a collision

        player.x == self.x
            && ((player.y as i32) < self.gap_y - half_size
                || player.y as i32 > self.gap_y + half_size)
        /*let does_x_match = player.x == self.x;
        // compare the player's Y coordinate with the obstacle's upper gap
        let player_above_gap = player.y < self.gap_y - half_size;
        let player_below_gap = player.y > self.gap_y + half_size;

        // If player's X coordinate matches that of the obstacle and the player's Y coord is either above or below the gap - collision has occurred
        does_x_match && (player_above_gap || player_below_gap)*/
    }
}

impl GameState for State {
    // the tick function should 'direct traffic' by managing the program flow depending on the current state
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::End => self.dead(ctx),
            GameMode::Playing => self.play(ctx),
        }
    }
    /*
    // ctx = short for context
    fn tick(&mut self, ctx: &mut BTerm) {
        // cls() clears the window
        ctx.cls();
        // x: 1 & y: 1, identifies where on the screen you want the String to appear - 0,0 is top left
        ctx.print(1, 1, "Hello, Bracket Terminal!");
    }
    */
}

// BError is a Result type enabling the use of the ? operator
fn main() -> BError {
    // bracket-lib library uses the BUILDER PATTERN, a Rust idiom relating to construction of complicated objects
    // It leverages function chaining to separate many options into individual function calls => more readable code than giant list of function parameters

    let context = BTermBuilder::new() // Builders start with an initial constructor that returns the builder - it is common to use frequently used starting points
        .with_font("../resources/flappy32.png", 32, 32) // provides a unique font of specified sizes
        .with_simple_console(SCREEN_WIDTH, SCREEN_HEIGHT, "../resources/flappy32.png")
        .with_fancy_console(SCREEN_WIDTH, SCREEN_HEIGHT, "../resources/flappy32.png")
        .with_title("Flappy Dragon Enhanced")
        .with_tile_dimensions(16, 16)   // changes the size of the window
        .build()?;

    // starts the game loop and links the engine to the State struct - bracket-lib then knows where the tick function is located
    main_loop(context, State::new())
}
