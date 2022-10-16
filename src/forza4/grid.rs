use {
    crate::key::Direction,
    game_arcade::clear_screen,
    colored::Colorize,
};

#[derive(PartialEq, Debug)]
struct Position {
    x: u8,
    y: u8,
}

#[derive(Debug)]
struct Cell {
    owner: String,
    position: Position,
}

pub struct Grid {
    pub current_player: String,
    pub players: Vec<String>,
    occupied_cells: Vec<Cell>,
    width: u8,
    height: u8,
    cursore: u8,
    n_move : i32,
}

impl Grid {
    pub fn new(players: Vec<String>, width: u8, height: u8) -> Grid {
        Grid {
            current_player: players[0].to_owned(),
            players,
            width,
            height,
            cursore: 0,
            occupied_cells: vec![],
            n_move : 0,
        }
    }

    pub fn print_grid(&self) {
        let mut griglia = String::from(" ");
        griglia.push_str("  ".repeat(self.cursore as usize).as_str());
        griglia.push_str("v\n");
        for y in 0..self.height {
            for x in 0..self.width {
                if self.is_free(&Position { x, y }) {
                    let cell = &self.occupied_cells.iter().filter(|cell| cell.position == Position {x, y } ).collect::<Vec<_>>() ;
                    griglia.push_str("|");
                    griglia.push(cell[0].owner.chars().next().unwrap());
                } else {
                    griglia.push_str("|_");
                }
            }
            griglia.push_str("|\n");
        }
        griglia.push_str(format!("È il turno di {}\n", self.current_player).as_str());
        clear_screen();
        print!("{}", griglia);
        // print!("{:#?}", self.occupied_cells);
        // println!("{:#?}" ,&self.occupied_cells.iter().filter(|cell| cell.owner == self.current_player ).collect::<Vec<_>>());
    }   

    fn is_free(&self, position: &Position) -> bool {
        self.occupied_cells
            .iter()
            .map(|c| &c.position)
            .position(|p| p == position)
            .is_some()
    }

    pub fn drop_coin(&mut self) {
        let x = self.cursore;
        let y = self
            .occupied_cells
            .iter()
            .map(|c| &c.position)
            .filter(|p| p.x == x)
            .map(|p| p.y - 1)
            .min()
            .unwrap_or(self.height - 1);
        // println!("{}", y);
        if let None = self
            .occupied_cells
            .iter()
            .map(|c| &c.position)
            .position(|position| position == &Position { x, y })
        {
            self.occupied_cells.push(Cell {
                owner: self.current_player.to_owned(),
                position: Position { x, y },
            });
        };
        self.occupied_cells.sort_by(|a, b| {
            if a.position.x == b.position.x {
                a.position.y.cmp(&b.position.y)
            } else {
                a.position.x.cmp(&b.position.x)
            }
        });
        self.win();
        self.next_player();
    }

    fn next_player(&mut self){
        let n_users = self.players.len() as i32;
        self.n_move =  self.n_move + 1 ;
        let new_user =  &self.players[(&self.n_move % n_users) as usize];
        self.current_player = new_user.to_owned() ;
    }

    pub fn move_cursor(&mut self, direction: Direction) {
        match direction {
            Direction::Right => {
                let range = 0..self.width - 1;
                if range.contains(&(self.cursore)) {
                    self.cursore += 1;
                }
            }
            // ------------------------------------
            Direction::Left => {
                let range = 0..self.width;
                if range.contains(&self.cursore) && self.cursore > 0 {
                    self.cursore -= 1;
                }
            }
        }
    }
    
    pub fn win(&mut self) {
        if &self.occupied_cells.len() >=  &4 {
            let current_user_cells = self.user_cells();
            for cell in &current_user_cells {
                let position: &Position = &cell.position;
                match &self.win_check(&current_user_cells, &position){
                    Some(i) => {self.print_grid() ;panic!("{} ha vinto", self.current_player)},
                    None => continue
                }
            }
        }
        else { }
    }
    fn user_cells(&self) -> Vec<&Cell>{
        self.occupied_cells.iter().filter(|cell| cell.owner == self.current_player ).collect::<Vec<_>>()
    }

    fn free_cells_in_user_vec(vec : &Vec<&Cell>, position: &Position) -> bool{
        vec
        .iter()
        .map(|c| &c.position)
        .position(|p| p == position)
        .is_some()
    }
    fn win_check(&self, vec : &Vec<&Cell>, position : &Position) -> Option<bool>{
        // orizontal win
        if !(1..=3).any(|i| !Grid::free_cells_in_user_vec(&vec,&Position { x : (position.x + i) , y : position.y})) {
            return Some(true);
        }
        // vertical win
        else if !(1..=3).any(|i| !Grid::free_cells_in_user_vec(&vec,&Position { x : (position.x ) , y : position.y + i})) {
            Some(true)
        }
        // diagonal \ win
        else if!(1..=3).any(|i| !Grid::free_cells_in_user_vec(&vec,&Position { x : (position.x +i ) , y : position.y + i})) {
            Some(true)
        }
        // diagonal / win   
        else if!(1..=3).any(|i| !Grid::free_cells_in_user_vec(&vec,&Position { x : (position.x +i ) , y : position.y - i})) {
            Some(true)
        }
        else {return None}
    }
}
