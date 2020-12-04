use std::{
    collections::{HashMap, HashSet, VecDeque},
    io::{self, BufRead},
    time::{Duration, Instant},
};

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

mod vec4 {
    use std::{
        fmt,
        ops::{Add, AddAssign, Sub, SubAssign},
    };

    #[derive(Eq, PartialEq, Copy, Clone, Hash)]
    pub struct Vec4 {
        pub x: i32,
        pub y: i32,
        pub z: i32,
        pub w: i32,
    }

    impl Vec4 {
        pub const fn new(x: i32, y: i32, z: i32, w: i32) -> Vec4 {
            Vec4 { x, y, z, w }
        }

        pub fn zero() -> Vec4 {
            Vec4 {
                x: 0,
                y: 0,
                z: 0,
                w: 0,
            }
        }

        pub fn magnitude2(self) -> f64 {
            (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w) as f64
        }

        pub fn magnitude(self) -> f64 {
            f64::sqrt(self.magnitude2())
        }

        pub fn is_non_neg(self) -> bool {
            self.x >= 0 && self.y >= 0 && self.z >= 0 && self.w >= 0
        }
    }

    impl fmt::Debug for Vec4 {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            write!(fmt, "({}, {}, {}, {})", self.x, self.y, self.z, self.w)
        }
    }

    impl Add for Vec4 {
        type Output = Vec4;

        fn add(self, rhs: Self) -> Self::Output {
            Vec4 {
                x: self.x + rhs.x,
                y: self.y + rhs.y,
                z: self.z + rhs.z,
                w: self.w + rhs.w,
            }
        }
    }

    impl AddAssign for Vec4 {
        fn add_assign(&mut self, rhs: Self) {
            self.x += rhs.x;
            self.y += rhs.y;
            self.z += rhs.z;
            self.w += rhs.w;
        }
    }

    impl Sub for Vec4 {
        type Output = Vec4;

        fn sub(self, rhs: Self) -> Self::Output {
            Vec4 {
                x: self.x - rhs.x,
                y: self.y - rhs.y,
                z: self.z - rhs.z,
                w: self.w - rhs.w,
            }
        }
    }

    impl SubAssign for Vec4 {
        fn sub_assign(&mut self, rhs: Self) {
            self.x -= rhs.x;
            self.y -= rhs.y;
            self.z -= rhs.z;
            self.w -= rhs.w;
        }
    }
}

use vec4::Vec4;

type Id = usize;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Action {
    Learn(Id),
    Brew(Id),
    Cast(Id),
    Rest,
    Wait,
}

impl ToString for Action {
    fn to_string(&self) -> String {
        match *self {
            Action::Learn(id) => format!("Learn {}", id),
            Action::Cast(id) => format!("Cast {}", id),
            Action::Brew(id) => format!("Brew {}", id),
            Action::Rest => "Rest".to_string(),
            Action::Wait => "Wait".to_string(),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
struct Spell {
    pub id: Id,
    pub delta: Vec4,
    pub is_repeatable: bool,
}

impl Spell {
    fn new(id: usize, delta: Vec4) -> Self {
        Self {
            id,
            delta,
            is_repeatable: !delta.is_non_neg(),
        }
    }

    fn can_be_afforded_by(self, inventory: Vec4) -> bool {
        (inventory + self.delta).is_non_neg()
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
struct TomeSpell {
    pub id: Id,
    pub delta: Vec4,
    pub tome_index: usize,
}

impl TomeSpell {
    fn new(id: Id, delta: Vec4, tome_index: usize) -> Self {
        Self {
            id,
            delta,
            tome_index,
        }
    }

    fn is_repeatable(&self) -> bool {
        !self.delta.is_non_neg()
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
struct Order {
    id: Id,
    price: i32,
    delta: Vec4,
}

impl Order {
    fn new(id: Id, price: i32, delta: Vec4) -> Self {
        Self { id, price, delta }
    }

    fn can_be_fulfilled_by(self, inventory: Vec4) -> bool {
        (inventory + self.delta).is_non_neg()
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
struct MagicTome {
    spells: Vec<TomeSpell>,
}

impl MagicTome {
    fn new(spells: Vec<TomeSpell>) -> Self {
        Self { spells }
    }

    fn add_spell(&mut self, spell: TomeSpell) {
        self.spells.push(spell)
    }

    fn remove_spell(&mut self, spell: TomeSpell) {
        if let Some(index) = self.spells.iter().position(|s| s.id == spell.id) {
            self.spells.remove(index);
        }
    }

    fn remove_spell_at(&mut self, index: usize) {
        self.spells.remove(index);
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
struct PlayerState {
    score: i32,
    inventory: Vec4,
    brew_count: i32,
    available_spells: Vec<usize>,
}

impl PlayerState {
    fn new(score: i32, brew_count: i32, inventory: Vec4, available_spells: Vec<usize>) -> Self {
        Self {
            score,
            brew_count,
            inventory,
            available_spells,
        }
    }

    fn evaluate(&self) -> f64 {
        let mut score = 0;
        let inv = self.inventory;
        score += self.score;
        score += inv.x + inv.y * 2 + inv.z * 3 + inv.w * 4;
        score as f64
    }

    fn can_afford_spell(&self, ingredients: Vec4) -> bool {
        (ingredients + self.inventory).is_non_neg()
    }

    // fn get_spell(&self, id: Id) -> Vec4 {
    //     match id {
    //         0..=3 => PRIMITIVE_SPELLS[id],
    //         4..=41 => TOME[id - 4],
    //         _ => unreachable!()
    //     }
    // }

    fn get_possible_actions(&self, game_state: &GameState, turn: usize) -> Vec<Action> {
        let mut actions = Vec::new();

        for order in game_state.orders.iter() {
            if order.can_be_fulfilled_by(self.inventory) {
                actions.push(Action::Brew(order.id));
            }
        }

        if turn < 5 {
            for (idx, spell) in game_state.tome.spells.iter().enumerate() {
                if self.inventory.x >= idx as i32 {
                    actions.push(Action::Learn(spell.id));
                }
            }
        }

        for &id in self.available_spells.iter() {
            if let Some(spell) = game_state.my_spells.iter().find(|s| s.id == id) {
                if self.can_afford_spell(spell.delta) {
                    actions.push(Action::Cast(id));
                }
            } else {
                unreachable!();
            }
        }

        let have_exhausted_spells = self.available_spells.len() != game_state.my_spells.len();

        if have_exhausted_spells {
            actions.push(Action::Rest);
        }
        // dbg!(&actions);
        actions
    }

    fn apply(&mut self, game_state: &mut GameState, action: Action) {
        match action {
            Action::Brew(id) => {
                let order_idx = game_state
                    .orders
                    .iter()
                    .position(|o| o.id == id && o.can_be_fulfilled_by(self.inventory))
                    .unwrap();
                let order = game_state.orders[order_idx];

                self.inventory += order.delta;
                self.score += order.price;
                self.brew_count += 1;

                game_state.orders.remove(order_idx);
            }
            Action::Cast(id) => {
                if let Some(spell) = game_state.my_spells.iter().find(|&s| s.id == id) {
                    if let Some(idx) = self.available_spells.iter().position(|&idx| idx == id) {
                        self.inventory += spell.delta;
                        self.available_spells.remove(idx);
                    } else {
                        panic!("tried removing already unavailable spell");
                    }
                }
                // dbg!(&self.me.spells);
            }
            Action::Learn(id) => {
                if let Some(spell_idx) = game_state.tome.spells.iter().position(|s| s.id == id) {
                    let spell = game_state.tome.spells[spell_idx];
                    let delta = Vec4::new(spell.tome_index as i32, 0, 0, 0);

                    if (self.inventory - delta).is_non_neg() {
                        let new_spell = Spell::new(spell.id + 1000, spell.delta);
                        game_state.my_spells.push(new_spell);
                        game_state.tome.remove_spell(spell)
                    }
                }
            }
            Action::Rest => {
                self.available_spells = game_state.my_spells.iter().map(|s| s.id).collect()
            }
            Action::Wait => {}
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
struct GameState {
    me: PlayerState,
    my_spells: Vec<Spell>,
    enemy: PlayerState,
    enemy_spells: Vec<Spell>,
    orders: Vec<Order>,
    tome: MagicTome,
}
//
impl GameState {
    fn new(
        me: PlayerState,
        my_spells: Vec<Spell>,
        enemy: PlayerState,
        enemy_spells: Vec<Spell>,
        orders: Vec<Order>,
        tome: MagicTome,
    ) -> Self {
        Self {
            me,
            my_spells,
            enemy,
            enemy_spells,
            orders,
            tome,
        }
    }

    fn get_possible_actions(&self) -> Vec<Action> {
        let mut actions = Vec::new();
        let mut can_use_rest = false;

        // for spell in self.tome.spells.iter() {
        //     if self.me.inventory.x >= spell.tome_index {
        //         actions.push(Action::Learn(spell.id));
        //     }
        // }

        for order in self.orders.iter() {
            if order.can_be_fulfilled_by(self.me.inventory) {
                actions.push(Action::Brew(order.id));
            }
        }

        // for spell in self.me.spells.iter() {
        //     if spell.is_castable {
        //         if spell.can_be_afforded_by(self.me.inventory) {
        //             actions.push(Action::Cast(spell.id));
        //         }
        //     } else {
        //         can_use_rest = true;
        //     }
        // }

        if can_use_rest {
            actions.push(Action::Rest);
        }
        // dbg!(&actions);
        actions
    }

    fn apply(&mut self, action: Action) {
        match action {
            Action::Brew(id) => {
                let order_idx = self
                    .orders
                    .iter()
                    .position(|o| o.id == id && o.can_be_fulfilled_by(self.me.inventory))
                    .unwrap();
                let order = self.orders[order_idx];

                self.me.inventory += order.delta;
                self.me.score += order.price;
                self.me.brew_count += 1;

                self.orders.remove(order_idx);
            }
            Action::Cast(id) => {
                if let Some(spell) = self.my_spells.iter().find(|&s| s.id == id) {
                    if let Some(idx) = self.me.available_spells.iter().position(|&idx| idx == id) {
                        self.me.inventory += spell.delta;
                        self.me.available_spells.remove(idx);
                    } else {
                        panic!("tried removing already unavailable spell");
                    }
                }
                // dbg!(&self.me.spells);
            }
            Action::Learn(id) => {
                if let Some(spell_idx) = self.tome.spells.iter().position(|s| s.id == id) {
                    let spell = self.tome.spells[spell_idx];
                    let delta = Vec4::new(spell.tome_index as i32, 0, 0, 0);

                    if (self.me.inventory - delta).is_non_neg() {
                        let new_spell = Spell::new(spell.id + 1000, spell.delta);
                        self.my_spells.push(new_spell);
                        self.tome.remove_spell(spell)
                    }
                }
            }
            Action::Rest => {
                self.me.available_spells = self.my_spells.iter().map(|s| s.id).collect()
            }
            Action::Wait => {}
        }
    }

    fn find_brewable_order(&self) -> Option<Order> {
        self.orders
            .iter()
            .find(|o| o.can_be_fulfilled_by(self.me.inventory))
            .cloned()
    }

    fn read_from_io() -> Self {
        // std::fs::read("C:/Users/Matharu/Desktop/Development/Workspace/codingame/fall-challenge-2020-rust/input.txt");
        // let file = std::fs::File::open("C:/Users/Matharu/Desktop/Development/Workspace/codingame/fall-challenge-2020-rust/input.txt").unwrap();
        // let mut reader = io::BufReader::new(file);

        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let action_count = parse_input!(input_line, i32); // the number of spells and recipes in play

        let mut orders = Vec::with_capacity(5);
        let mut my_spells = Vec::new();
        let mut enemy_spells = Vec::new();
        let mut tome = Vec::new();

        for _ in 0..action_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(' ').collect::<Vec<_>>();
            let action_id = parse_input!(inputs[0], Id); // the unique ID of this spell or recipe
            let action_type = inputs[1].trim().to_string(); // in the first league: BREW; later: CAST, OPPONENT_CAST, LEARN, BREW

            let x = parse_input!(inputs[2], i32); // tier-0 ingredient change
            let y = parse_input!(inputs[3], i32); // tier-1 ingredient change
            let z = parse_input!(inputs[4], i32); // tier-2 ingredient change
            let w = parse_input!(inputs[5], i32); // tier-3 ingredient change
            let delta = Vec4::new(x, y, z, w);
            let price = parse_input!(inputs[6], i32); // the price in rupees if this is a potion
            let tome_index = parse_input!(inputs[7], i32); // in the first two leagues: always 0; later: the index in the tome if this is a tome spell, equal to the read-ahead tax; For brews, this is the value of the current urgency bonus
            let tax_count = parse_input!(inputs[8], i32); // in the first two leagues: always 0; later: the amount of taxed tier-0 ingredients you gain from learning this spell; For brews, this is how many times you can still gain an urgency bonus
            let castable = parse_input!(inputs[9], i32) == 1; // in the first league: always 0; later: 1 if this is a castable player spell
            let repeatable = parse_input!(inputs[10], i32) == 1; // for the first two leagues: always 0; later: 1 if this is a repeatable player spell

            match action_type.as_str() {
                "BREW" => {
                    orders.push(Order::new(action_id, price, delta));
                }
                "CAST" => {
                    my_spells.push(Spell::new(action_id, delta));
                }
                "OPPONENT_CAST" => {
                    enemy_spells.push(Spell::new(action_id, delta));
                }
                "LEARN" => {
                    tome.push(TomeSpell::new(action_id, delta, tome_index as usize));
                }
                _ => {}
            }
        }

        let read_player = |spells: &Vec<Spell>| {
            let mut input_line = String::new();
            // io::stdin()
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(' ').collect::<Vec<_>>();
            let x = parse_input!(inputs[0], i32); // tier-0 ingredients in inventory
            let y = parse_input!(inputs[1], i32);
            let z = parse_input!(inputs[2], i32);
            let w = parse_input!(inputs[3], i32);
            let score = parse_input!(inputs[4], i32); // amount of rupees
            PlayerState::new(
                score,
                0,
                Vec4::new(x, y, z, w),
                spells.iter().map(|s| s.id).collect(),
            )
        };

        let me = read_player(&my_spells);
        let enemy = read_player(&enemy_spells);
        GameState::new(
            me,
            my_spells,
            enemy,
            enemy_spells,
            orders,
            MagicTome::new(tome),
        )
    }
}

#[derive(Copy, Clone)]
struct Bot;

impl Bot {
    fn new() -> Self {
        Self {}
    }

    fn bfs(self, start_instant: &Instant, game_state: &mut GameState, turn: usize) -> Vec<Action> {
        const MAX_DURATION: Duration = Duration::from_millis(40);
        let state = game_state.me.clone();

        let mut queue = VecDeque::<PlayerState>::new();
        let mut visited = HashSet::<PlayerState>::new();

        let mut predecessor = HashMap::<PlayerState, PlayerState>::new();
        let mut pred_action = HashMap::<PlayerState, Action>::new();

        let initial_state = state.clone();
        queue.push_back(state.clone());
        pred_action.insert(state.clone(), Action::Wait);
        visited.insert(state.clone());
        let mut iterations = 0;
        let mut best_state = Option::None;
        let mut best_score = f64::MIN;

        while let Some(current_state) = queue.pop_front() {
            if start_instant.elapsed() > MAX_DURATION {
                let mut path = Vec::<Action>::new();
                let mut curr_state = best_state.expect("best state missing");
                // let mut curr_state = current_state;
                // dbg!(&curr_state);

                while curr_state != initial_state {
                    let action = pred_action.get(&curr_state).expect("pred action not found");
                    let last_state = predecessor.get(&curr_state).expect("prev state not found");
                    path.push(*action);
                    curr_state = last_state.clone();
                }

                println!("{} game states visited", iterations);
                return path;
            }

            let curr_state = &current_state;

            let score = curr_state.evaluate();
            if score > best_score {
                best_score = score;
                best_state = Some(curr_state.clone())
            }

            for action in current_state.get_possible_actions(&game_state, turn) {
                let mut next = curr_state.clone();
                next.apply(game_state, action);
                iterations += 1;

                if !visited.contains(&next) {
                    queue.push_back(next.clone());
                    pred_action.entry(next.clone()).or_insert(action);
                    predecessor
                        .entry(next.clone())
                        .or_insert(curr_state.clone());
                    visited.insert(next.clone());
                }
            }
        }

        Vec::new()
    }

    fn think(self, start_instant: &Instant, state: &mut GameState, turn: usize) -> Action {
        if let Some(order) = state.find_brewable_order() {
            return Action::Brew(order.id);
        }

        let actions: Vec<Action> = self
            .bfs(start_instant, state, turn)
            .into_iter()
            .rev()
            .collect();

        for action in actions.iter() {
            println!("{}", action.to_string());
        }

        if let Some(&action) = actions.first() {
            return action;
        }

        // println!("Time took: {} ms", start_instant.elapsed().as_millis());

        println!("no actions computed, taking first possible action");
        Action::Wait
        // state.get_possible_actions().first().cloned().unwrap()
    }
}

fn main() {
    let bot = Bot::new();

    let state = GameState::read_from_io();
    let mut turn = 0;
    let mut total_duration = Duration::new(0, 0);

    for _ in 0..10 {
        let mut turn_state = state.clone();
        // let turn_state = State::read_from_io();

        let start_instant = Instant::now();
        let action = bot.think(&start_instant, &mut turn_state, turn);
        total_duration += start_instant.elapsed();
        turn += 1;
        println!("Action {}", action.to_string());
    }

    println!("Average time taken: {:?}", total_duration / 100)
}
