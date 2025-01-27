// Ajuna Node
// Copyright (C) 2022 BlogaTech AG

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use crate::*;

const ALICE: u8 = 11;
const BOB: u8 = 22;
const CHARLIE: u8 = 33;
const TEST_COORDINATES: Coordinates = Coordinates::new(0, 0);

#[test]
fn should_create_a_new_board() {
    fn is_empty(board: &Board) -> bool {
        let mut empty = true;
        for row in board.cells {
            for cell in row {
                if cell != Cell::Empty {
                    empty = false;
                }
            }
        }
        empty
    }

    let board = Board::new();
    assert_eq!(board.cells.len() as u8, BOARD_HEIGHT);
    assert_eq!(board.cells[0].len() as u8, BOARD_WIDTH);
    assert!(is_empty(&board))
}

#[test]
fn board_cell_can_be_changed() {
    let mut board = Board::new();
    let coords = Coordinates { row: 5, col: 5 };

    assert_eq!(
        board.get_cell(&coords),
        Cell::Empty,
        "Cell should be empty before change."
    );
    board.update_cell(coords, Cell::Block);
    assert_eq!(
        board.get_cell(&coords),
        Cell::Block,
        "Cell should had changed."
    );
}

#[test]
fn should_create_new_game() {
    let game_state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    let computed_from_initial_seed = 46_384;
    assert_eq!(game_state.seed, computed_from_initial_seed);
    assert_eq!(
        game_state.phase,
        GamePhase::Bomb,
        "The game should start in bomb phase"
    );
    assert_eq!(game_state.winner, None, "No player should have won yet");
    assert_eq!(game_state.next_player, ALICE);
    assert_eq!(game_state.bombs.len(), NUM_OF_PLAYERS);
    assert_eq!(
        game_state.get_player_bombs(&ALICE),
        Some(NUM_OF_BOMBS_PER_PLAYER),
    );
    assert_eq!(
        game_state.get_player_bombs(&BOB),
        Some(NUM_OF_BOMBS_PER_PLAYER),
    );
    assert!(
        game_state.is_player_in_game(&ALICE),
        "Player Alice should be in the game"
    );
    assert!(
        game_state.is_player_in_game(&BOB),
        "Player Bob should be in the game"
    );
    assert_eq!(game_state.last_move, None);

    assert_eq!(game_state.get_player_score(&ALICE), Score::default());
    assert_eq!(game_state.get_player_score(&BOB), Score::default());
}

#[test]
fn should_create_new_game_with_random_blocks() {
    let blocks = |board: Board| -> u8 {
        let mut block_count = 0;
        board.cells.iter().for_each(|row| {
            row.iter().for_each(|cell| {
                if let Cell::Block = cell {
                    block_count += 1;
                }
            })
        });
        block_count
    };

    let (mut seed_1, mut seed_2) = (123, 456);
    for _ in 0..20 {
        let game_1 = Game::new_game(ALICE, BOB, Some(seed_1));
        let game_2 = Game::new_game(ALICE, BOB, Some(seed_2));
        assert_ne!(game_1.board, game_2.board);
        assert_eq!(blocks(game_1.board), NUM_OF_BLOCKS);
        assert_eq!(blocks(game_2.board), NUM_OF_BLOCKS);
        assert_ne!(seed_1, game_1.seed, "seed 1 should be updated");
        assert_ne!(seed_2, game_2.seed, "seed 2 should be updated");
        seed_1 = game_1.seed;
        seed_2 = game_2.seed;
    }
}

#[test]
fn should_create_new_game_with_deterministic_blocks_with_fixed_seed() {
    let seed = 7357;
    for _ in 0..20 {
        let game_1 = Game::new_game(ALICE, BOB, Some(seed));
        let game_2 = Game::new_game(ALICE, BOB, Some(seed));
        assert_eq!(game_1.board, game_2.board);
    }
}

#[test]
fn a_player_cannot_drop_bomb_in_play_phase() {
    let mut game_state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    game_state.phase = GamePhase::Play;
    let result = Game::drop_bomb(game_state, TEST_COORDINATES, ALICE);
    assert_eq!(result, Err(GameError::DroppedBombOutsideBombPhase));
}

#[test]
fn a_player_cannot_drop_bomb_if_already_dropped_all() {
    for n in 0..NUM_OF_BOMBS_PER_PLAYER {
        let mut game_state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
        game_state.bombs = [(ALICE, 0), (BOB, n)];
        assert_eq!(
            Game::drop_bomb(game_state, TEST_COORDINATES, ALICE),
            Err(GameError::NoMoreBombsAvailable)
        );

        game_state.bombs = [(ALICE, n), (BOB, 0)];
        assert_eq!(
            Game::drop_bomb(game_state, TEST_COORDINATES, BOB),
            Err(GameError::NoMoreBombsAvailable)
        );
    }
}

#[test]
fn a_player_cannot_drop_bomb_if_game_already_finished() {
    let mut game_state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    game_state.winner = Some(ALICE);
    assert_eq!(
        Game::drop_bomb(game_state, TEST_COORDINATES, BOB),
        Err(GameError::GameAlreadyFinished),
    )
}

#[test]
fn dropping_bomb_should_not_update_last_move() {
    let mut game_state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    game_state.board.update_cell(TEST_COORDINATES, Cell::Empty);

    assert!(Game::drop_bomb(game_state, TEST_COORDINATES, ALICE).is_ok());
    assert_eq!(game_state.last_move, None);
}

#[test]
fn a_player_drops_a_bomb() {
    let mut game_state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    game_state.board.update_cell(TEST_COORDINATES, Cell::Empty);

    let player_bombs = game_state.get_player_bombs(&ALICE).unwrap();
    assert_eq!(player_bombs, NUM_OF_BOMBS_PER_PLAYER);

    let game_state = Game::drop_bomb(game_state, TEST_COORDINATES, ALICE).unwrap();
    assert_eq!(
        game_state.get_player_bombs(&ALICE).unwrap(),
        player_bombs - 1,
        "The player should have one bomb less available for dropping"
    );
    assert_eq!(
        game_state.board.get_cell(&TEST_COORDINATES),
        Cell::Bomb([Some(game_state.player_index(&ALICE)), None])
    )
}

#[test]
fn a_cell_can_hold_one_or_more_bombs_from_different_players() {
    let mut game_state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    let (alice_index, bob_index) = (
        game_state.player_index(&ALICE),
        game_state.player_index(&BOB),
    );
    game_state.board.update_cell(TEST_COORDINATES, Cell::Empty);

    let drop_bomb_result = Game::drop_bomb(game_state, TEST_COORDINATES, ALICE);
    assert!(drop_bomb_result.is_ok());
    game_state = drop_bomb_result.unwrap();

    assert_eq!(
        game_state.board.get_cell(&TEST_COORDINATES),
        Cell::Bomb([Some(alice_index), None])
    );

    let drop_bomb_result = Game::drop_bomb(game_state, TEST_COORDINATES, BOB);
    assert!(drop_bomb_result.is_ok());
    game_state = drop_bomb_result.unwrap();

    assert_eq!(
        game_state.board.get_cell(&TEST_COORDINATES),
        Cell::Bomb([Some(alice_index), Some(bob_index)])
    );
}

#[test]
fn a_cell_cannot_hold_more_than_allowed_number_of_bombs() {
    let mut game_state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    let (alice_index, bob_index) = (
        game_state.player_index(&ALICE),
        game_state.player_index(&BOB),
    );
    let bombed_cell = Cell::Bomb([Some(bob_index), Some(alice_index)]);
    game_state.board.update_cell(TEST_COORDINATES, bombed_cell);
    assert_eq!(
        Game::drop_bomb(game_state, TEST_COORDINATES, ALICE),
        Err(GameError::InvalidBombPosition)
    );
    assert_eq!(
        Game::drop_bomb(game_state, TEST_COORDINATES, BOB),
        Err(GameError::InvalidBombPosition)
    );
}

#[test]
fn a_bomb_cannot_be_placed_in_a_cell_occupied_by_a_block() {
    let mut game_state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    game_state.board.update_cell(TEST_COORDINATES, Cell::Block);
    assert_eq!(
        Game::drop_bomb(game_state, TEST_COORDINATES, ALICE),
        Err(GameError::InvalidBombPosition)
    );
    assert_eq!(
        Game::drop_bomb(game_state, TEST_COORDINATES, BOB),
        Err(GameError::InvalidBombPosition)
    );
}

#[test]
fn a_player_cannot_place_more_than_one_bomb_in_a_cell() {
    let mut game_state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    let alice_index = game_state.player_index(&ALICE);
    game_state.board.update_cell(TEST_COORDINATES, Cell::Empty);

    // Drop the first bomb
    let drop_bomb_result = Game::drop_bomb(game_state, TEST_COORDINATES, ALICE);
    assert!(
        drop_bomb_result.is_ok(),
        "Dropping the first bomb should be OK"
    );
    game_state = drop_bomb_result.unwrap();

    assert_eq!(
        game_state.board.get_cell(&TEST_COORDINATES),
        Cell::Bomb([Some(alice_index), None])
    );

    // Drop the second bomb
    let drop_bomb_result = Game::drop_bomb(game_state, TEST_COORDINATES, ALICE);
    assert_eq!(drop_bomb_result, Err(GameError::InvalidBombPosition));
}

#[test]
fn a_game_can_change_game_phase() {
    let game_state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    assert_eq!(game_state.phase, GamePhase::Bomb);
    let game_state = Game::change_game_phase(game_state, GamePhase::Play);
    assert_eq!(game_state.phase, GamePhase::Play);
    let game_state = Game::change_game_phase(game_state, GamePhase::Bomb);
    assert_eq!(game_state.phase, GamePhase::Bomb);
}

#[test]
fn a_player_cannot_drop_a_stone_in_bomb_phase() {
    let state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    assert_eq!(state.phase, GamePhase::Bomb);
    assert_eq!(
        Game::drop_stone(state, BOB, Side::North, 0),
        Err(GameError::DroppedStoneOutsidePlayPhase)
    );
}

#[test]
fn a_player_cannot_drop_a_stone_out_of_turn() {
    let mut state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    state.phase = GamePhase::Play;
    let drop_stone_result = Game::drop_stone(state, BOB, Side::North, 0);
    assert_eq!(drop_stone_result, Err(GameError::NotPlayerTurn));
}

#[test]
fn a_player_cannot_drop_stone_if_game_already_finished() {
    let mut game_state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    game_state.phase = GamePhase::Play;
    game_state.winner = Some(BOB);
    assert_eq!(
        Game::drop_stone(game_state, ALICE, Side::East, 1),
        Err(GameError::GameAlreadyFinished),
    )
}

#[test]
fn player_turn_changes_after_dropping_stone() {
    let mut state = Game::new_game(CHARLIE, BOB, Some(INITIAL_SEED));
    for i in 0..BOARD_WIDTH {
        state.board.update_cell(Coordinates::new(i, 0), Cell::Empty);
    }
    state.phase = GamePhase::Play;
    let drop_stone_result = Game::drop_stone(state, CHARLIE, Side::North, 0);
    assert!(drop_stone_result.is_ok());
    let state = drop_stone_result.unwrap();

    let drop_stone_result = Game::drop_stone(state, CHARLIE, Side::North, 0);
    assert_eq!(drop_stone_result, Err(GameError::NotPlayerTurn));

    let drop_stone_result = Game::drop_stone(state, BOB, Side::North, 0);
    assert!(drop_stone_result.is_ok());
}

#[test]
fn last_move_changes_after_dropping_stone() {
    let mut state = Game::new_game(BOB, ALICE, Some(INITIAL_SEED));
    state.phase = GamePhase::Play;
    assert_eq!(state.last_move, None);

    for (player, side, position) in [
        (BOB, Side::West, 2),
        (BOB, Side::East, 1),
        (BOB, Side::North, 6),
        (BOB, Side::South, 8),
    ] {
        let state = Game::drop_stone(state, player, side, position).unwrap();
        assert_eq!(state.last_move, Some(LastMove::new(player, side, position)));
    }
}

#[test]
fn a_stone_dropped_on_a_stone() {
    let mut state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    let (alice_index, bob_index) = (state.player_index(&ALICE), state.player_index(&BOB));

    let o = Cell::Empty;
    let x = Cell::Stone(bob_index);
    let cells = [
        [o, x, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
    ];

    state.board.cells = cells;
    state.phase = GamePhase::Play;

    let state = Game::drop_stone(state, ALICE, Side::West, 0).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 0, col: 0 }),
        Cell::Stone(alice_index)
    );
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 0, col: 1 }),
        Cell::Stone(bob_index)
    );
}

#[test]
fn a_stone_cannot_be_dropped_at_bounds() {
    let mut state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    state.phase = GamePhase::Play;

    let mut state_with_stones_at_bounds = state;
    let o = Cell::Empty;
    let x = Cell::Stone(state.player_index(&BOB));
    state_with_stones_at_bounds.board.cells = [
        [x, x, x, x, x, x, x, x, x, x],
        [x, o, o, o, o, o, o, o, o, x],
        [x, o, o, o, o, o, o, o, o, x],
        [x, o, o, o, o, o, o, o, o, x],
        [x, o, o, o, o, o, o, o, o, x],
        [x, o, o, o, o, o, o, o, o, x],
        [x, o, o, o, o, o, o, o, o, x],
        [x, o, o, o, o, o, o, o, o, x],
        [x, o, o, o, o, o, o, o, o, x],
        [x, x, x, x, x, x, x, x, x, x],
    ];

    let mut state_with_blocks_at_bounds = state;
    let b = Cell::Block;
    state_with_blocks_at_bounds.board.cells = [
        [b, b, b, b, b, b, b, b, b, b],
        [b, o, o, o, o, o, o, o, o, b],
        [b, o, o, o, o, o, o, o, o, b],
        [b, o, o, o, o, o, o, o, o, b],
        [b, o, o, o, o, o, o, o, o, b],
        [b, o, o, o, o, o, o, o, o, b],
        [b, o, o, o, o, o, o, o, o, b],
        [b, o, o, o, o, o, o, o, o, b],
        [b, o, o, o, o, o, o, o, o, b],
        [b, b, b, b, b, b, b, b, b, x],
    ];

    for state in [state_with_stones_at_bounds, state_with_blocks_at_bounds] {
        // left -> right check, dropping stones from top and bottom
        for position in 0..BOARD_WIDTH {
            assert_eq!(
                Game::drop_stone(state, ALICE, Side::North, position),
                Err(GameError::InvalidStonePosition)
            );
            assert_eq!(
                Game::drop_stone(state, ALICE, Side::South, position),
                Err(GameError::InvalidStonePosition)
            );
        }

        // top -> bottom check, dropping stones from left and right
        for position in 0..BOARD_HEIGHT {
            assert_eq!(
                Game::drop_stone(state, ALICE, Side::West, position),
                Err(GameError::InvalidStonePosition)
            );
            assert_eq!(
                Game::drop_stone(state, ALICE, Side::East, position),
                Err(GameError::InvalidStonePosition)
            );
        }
    }
}

#[test]
fn a_stone_dropped_from_north_side_should_move_until_it_reaches_an_obstacle() {
    let o = Cell::Empty;
    let b = Cell::Block;
    let cells = [
        [o, o, o, b, o, o, o, o, o, o],
        [o, o, b, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, b, o, o, o, o, o, o, o, o],
    ];

    let mut state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    state.board.cells = cells;
    state.phase = GamePhase::Play;

    let state = Game::drop_stone(state, ALICE, Side::North, 0).unwrap();
    let (alice_index, bob_index) = (state.player_index(&ALICE), state.player_index(&BOB));
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 9, col: 0 }),
        Cell::Stone(alice_index)
    );
    let state = Game::drop_stone(state, BOB, Side::North, 1).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 8, col: 1 }),
        Cell::Stone(bob_index)
    );
    let state = Game::drop_stone(state, ALICE, Side::North, 2).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 0, col: 2 }),
        Cell::Stone(alice_index)
    );
    assert_eq!(
        Game::drop_stone(state, BOB, Side::North, 3).unwrap_err(),
        GameError::InvalidStonePosition
    );
}

#[test]
fn a_stone_dropped_from_south_side_should_move_until_it_reaches_an_obstacle() {
    let o = Cell::Empty;
    let b = Cell::Block;

    let cells = [
        [o, b, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, b, o, o, o, o, o, o, o],
        [o, o, o, b, o, o, o, o, o, o],
    ];

    let mut state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    let (alice_index, bob_index) = (state.player_index(&ALICE), state.player_index(&BOB));
    state.board.cells = cells;
    state.phase = GamePhase::Play;

    let state = Game::drop_stone(state, ALICE, Side::South, 0).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 0, col: 0 }),
        Cell::Stone(alice_index)
    );
    let state = Game::drop_stone(state, BOB, Side::South, 1).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 1, col: 1 }),
        Cell::Stone(bob_index)
    );
    let state = Game::drop_stone(state, ALICE, Side::South, 2).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 9, col: 2 }),
        Cell::Stone(alice_index)
    );
    assert_eq!(
        Game::drop_stone(state, BOB, Side::South, 3).unwrap_err(),
        GameError::InvalidStonePosition
    );
}

#[test]
fn a_stone_dropped_from_east_side_should_move_until_it_reaches_an_obstacle() {
    let o = Cell::Empty;
    let b = Cell::Block;

    let cells = [
        [o, o, o, o, o, o, o, o, o, o],
        [b, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, b, o],
        [o, o, o, o, o, o, o, o, o, b],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
    ];

    let mut state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    let (alice_index, bob_index) = (state.player_index(&ALICE), state.player_index(&BOB));
    state.board.cells = cells;
    state.phase = GamePhase::Play;

    let state = Game::drop_stone(state, ALICE, Side::East, 0).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 0, col: 0 }),
        Cell::Stone(alice_index)
    );
    let state = Game::drop_stone(state, BOB, Side::East, 1).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 1, col: 1 }),
        Cell::Stone(bob_index)
    );
    let state = Game::drop_stone(state, ALICE, Side::East, 2).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 2, col: 9 }),
        Cell::Stone(alice_index)
    );
    assert_eq!(
        Game::drop_stone(state, BOB, Side::East, 3).unwrap_err(),
        GameError::InvalidStonePosition
    );
}

#[test]
fn a_stone_dropped_from_west_side_should_move_until_it_reaches_an_obstacle() {
    let o = Cell::Empty;
    let b = Cell::Block;

    let cells = [
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, b],
        [o, b, o, o, o, o, o, o, o, o],
        [b, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
    ];

    let mut state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    state.board.cells = cells;
    state.phase = GamePhase::Play;

    let state = Game::drop_stone(state, ALICE, Side::West, 0).unwrap();
    let (alice_index, bob_index) = (state.player_index(&ALICE), state.player_index(&BOB));
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 0, col: 9 }),
        Cell::Stone(alice_index)
    );
    let state = Game::drop_stone(state, BOB, Side::West, 1).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 1, col: 8 }),
        Cell::Stone(bob_index)
    );
    let state = Game::drop_stone(state, ALICE, Side::West, 2).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 2, col: 0 }),
        Cell::Stone(alice_index)
    );
    assert_eq!(
        Game::drop_stone(state, BOB, Side::West, 3).unwrap_err(),
        GameError::InvalidStonePosition
    );
}

#[test]
fn a_stone_should_explode_a_bomb_when_passing_through() {
    let mut state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    let (alice_index, bob_index) = (state.player_index(&ALICE), state.player_index(&BOB));
    let o = Cell::Empty;
    let b = Cell::Bomb([Some(alice_index), Some(bob_index)]);
    let x = Cell::Stone(alice_index);
    let l = Cell::Block;
    state.board.cells = [
        [o, o, o, o, o, o, o, o, b, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, b, o, o, o, o, o, o],
        [o, o, o, o, x, b, l, o, o, o],
        [o, o, o, o, b, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [b, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, b],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, b, o, o, o, o, o],
    ];
    state.phase = GamePhase::Play;

    let dropping_stone_result = Game::drop_stone(state, ALICE, Side::North, 5);
    assert!(dropping_stone_result.is_ok());
    state = dropping_stone_result.unwrap();

    // Stone in position 3,4 should be destroyed.
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 3, col: 4 }),
        Cell::Empty
    );

    // Bomb in position 4,4 should be destroyed.
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 4, col: 4 }),
        Cell::Empty
    );

    // Block in position 3,6 should not be destroyed.
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 3, col: 6 }),
        Cell::Block
    );

    // Bomb in position 2,3 should not be destroyed.
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 2, col: 3 }),
        Cell::Bomb([Some(alice_index), Some(bob_index)])
    );

    let dropping_stone_result = Game::drop_stone(state, BOB, Side::North, 8);
    assert!(dropping_stone_result.is_ok());
    state = dropping_stone_result.unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 0, col: 8 }),
        Cell::Empty
    );

    let dropping_stone_result = Game::drop_stone(state, ALICE, Side::East, 7);
    assert!(dropping_stone_result.is_ok());
    state = dropping_stone_result.unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 7, col: 9 }),
        Cell::Empty
    );

    let dropping_stone_result = Game::drop_stone(state, BOB, Side::South, 4);
    assert!(dropping_stone_result.is_ok());
    state = dropping_stone_result.unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 9, col: 4 }),
        Cell::Empty
    );

    let dropping_stone_result = Game::drop_stone(state, ALICE, Side::South, 4);
    assert!(dropping_stone_result.is_ok());
    state = dropping_stone_result.unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 4, col: 4 }),
        Cell::Empty
    );
}

#[test]
fn a_player_wins_when_has_stones_in_three_squares() {
    let mut state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    let alice_index = state.player_index(&ALICE);
    let o = Cell::Empty;
    let s = Cell::Stone(alice_index);
    state.board.cells = [
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, s, s, o, o, o, o, o, o],
        [o, o, s, s, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, s, s, o, o, o],
        [o, o, o, o, o, s, s, o, o, o],
        [o, o, o, s, s, o, o, o, o, o],
        [o, o, o, s, s, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
    ];

    state = Game::check_winner_player(state);
    assert_eq!(state.winner, Some(ALICE));
}

#[test]
fn a_player_wins_when_has_stones_in_three_squares_with_overlap() {
    let mut state = Game::new_game(CHARLIE, BOB, Some(INITIAL_SEED));
    let winner_index = state.player_index(&BOB);
    let o = Cell::Empty;
    let w = Cell::Stone(winner_index);
    state.board.cells = [
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, w, w, o, o, o, o],
        [o, o, o, w, w, w, o, o, o, o],
        [o, o, o, w, w, w, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
    ];

    state = Game::check_winner_player(state);
    assert_eq!(state.winner, Some(BOB));
}

#[test]
fn no_player_wins_if_stones_are_not_in_four_squares() {
    let mut state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    let o = Cell::Empty;
    let b = Cell::Block;
    let r = Cell::Stone(state.player_index(&ALICE));
    let m = Cell::Stone(state.player_index(&BOB));
    state.board.cells = [
        [o, r, o, o, o, o, o, o, m, o],
        [m, o, o, o, o, m, o, o, o, o],
        [m, o, r, r, m, m, m, o, o, o],
        [b, o, o, o, o, m, m, o, o, o],
        [m, m, o, o, r, o, o, o, o, o],
        [m, m, b, m, o, r, o, o, o, o],
        [o, o, o, o, b, o, m, o, o, o],
        [o, o, r, o, o, o, o, r, o, o],
        [r, r, r, o, o, o, o, o, o, o],
        [r, r, r, o, o, o, o, o, o, o],
    ];

    state = Game::check_winner_player(state);
    assert!(state.winner.is_none(), "No player should have won");
}

#[test]
fn a_stone_dropped_should_increase_score() {
    let o = Cell::Empty;
    let mut state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));

    state.board.cells = [
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
    ];

    state.phase = GamePhase::Play;

    assert_eq!(state.get_player_score(&ALICE), Score::default());
    assert_eq!(state.get_player_score(&BOB), Score::default());

    let mut state = Game::drop_stone(state, ALICE, Side::West, 0).unwrap();
    assert_eq!(
        state.get_player_score(&ALICE),
        Score::default() + NB_POINT_STONE
    );
    assert_eq!(state.get_player_score(&BOB), Score::default());

    state = Game::drop_stone(state, BOB, Side::North, 0).unwrap();
    assert_eq!(
        state.get_player_score(&ALICE),
        Score::default() + NB_POINT_STONE
    );
    assert_eq!(
        state.get_player_score(&BOB),
        Score::default() + NB_POINT_STONE
    );
}

#[test]
fn get_nb_ennemy_stones_destroy_by_player_bomb() {
    let mut state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    let (alice_index, bob_index) = (state.player_index(&ALICE), state.player_index(&BOB));

    let o = Cell::Empty;
    let b = Cell::Block;
    let a = Cell::Bomb([Some(alice_index), None]);
    let s = Cell::Stone(bob_index);

    state.board.cells = [
        [o, o, o, o, o, o, o, o, o, o],
        [s, o, s, o, o, o, o, o, o, o],
        [o, a, s, o, o, o, o, o, o, o],
        [b, b, b, b, o, o, o, o, o, o],
        [b, o, o, o, o, o, o, o, o, o],
        [b, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, b, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
    ];

    state.phase = GamePhase::Play;

    // State before the explosion
    assert_eq!(state.board.get_cell(&Coordinates { row: 1, col: 0 }), s);
    assert_eq!(state.board.get_cell(&Coordinates { row: 1, col: 1 }), o);
    assert_eq!(state.board.get_cell(&Coordinates { row: 1, col: 2 }), s);
    assert_eq!(state.board.get_cell(&Coordinates { row: 2, col: 0 }), o);
    assert_eq!(state.board.get_cell(&Coordinates { row: 2, col: 1 }), a);
    assert_eq!(state.board.get_cell(&Coordinates { row: 2, col: 2 }), s);

    state = Game::drop_stone(state, ALICE, Side::North, 1).unwrap();

    // State after the explosion
    assert_eq!(state.board.get_cell(&Coordinates { row: 1, col: 0 }), o);
    assert_eq!(state.board.get_cell(&Coordinates { row: 1, col: 1 }), o);
    assert_eq!(state.board.get_cell(&Coordinates { row: 1, col: 2 }), o);
    assert_eq!(state.board.get_cell(&Coordinates { row: 2, col: 0 }), o);
    assert_eq!(state.board.get_cell(&Coordinates { row: 2, col: 1 }), o);
    assert_eq!(state.board.get_cell(&Coordinates { row: 2, col: 2 }), o);

    assert_eq!(state.get_player_score(&ALICE), 4);
    assert_eq!(state.get_player_score(&BOB), 0);
}

#[test]
fn destroy_enemy_stone_should_increase_player_score() {
    let mut state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    let (alice_index, bob_index) = (state.player_index(&ALICE), state.player_index(&BOB));
    let mut alice_score = Score::default();
    let mut bob_score = Score::default();

    let o = Cell::Empty;
    let b = Cell::Block;

    let m = Cell::Bomb([Some(alice_index), None]);
    let n = Cell::Bomb([Some(bob_index), None]);

    let y = Cell::Stone(alice_index);
    let z = Cell::Stone(bob_index);

    state.board.cells = [
        [o, o, o, o, o, o, o, o, b, o],
        [b, o, m, z, o, o, o, o, z, m],
        [b, o, o, b, b, b, b, o, z, o],
        [b, o, o, o, o, o, o, o, o, o],
        [b, o, o, o, o, o, o, o, o, o],
        [b, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, b, o, o, o],
        [o, o, b, o, o, o, o, o, y, o],
        [o, n, o, o, o, o, o, n, o, o],
        [o, y, o, o, o, o, o, o, b, o],
    ];

    state.phase = GamePhase::Play;

    // Alice drop a stone on (1, 9) and destroyed her bomb on (1, 9) which destroy Bob stones on (1, 8) and (2, 8)
    state = Game::drop_stone(state, ALICE, Side::East, 1).unwrap();
    alice_score += NB_POINT_STONE + 2 * NB_POINT_ENEMY_STONE_DESTROYED;

    // Bob drop a stone (9, 7) and destroy his bomb on (8, 7) which destroy Alice stone on (7, 8)
    state = Game::drop_stone(state, BOB, Side::South, 7).unwrap();
    bob_score += NB_POINT_STONE + NB_POINT_ENEMY_STONE_DESTROYED;

    // Alice drop a stone on (0, 2) and destroyed her bomb on (1, 2) which destroy Bob stone on (0, 3)
    state = Game::drop_stone(state, ALICE, Side::North, 2).unwrap();
    alice_score += NB_POINT_STONE + NB_POINT_ENEMY_STONE_DESTROYED;

    // Bob drop a stone (8, 0) and destroy his bomb on (8, 1) which destroy Alice stone on (9, 1)
    state = Game::drop_stone(state, BOB, Side::West, 8).unwrap();
    bob_score += NB_POINT_STONE + NB_POINT_ENEMY_STONE_DESTROYED;

    assert_eq!(state.get_player_score(&ALICE), alice_score);
    assert_eq!(state.get_player_score(&BOB), bob_score);
}

#[test]
fn destroy_its_own_stone_should_not_increase_player_score() {
    let o = Cell::Empty;
    let b = Cell::Block;

    let mut state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    state.board.cells = [
        [o, o, o, o, o, o, o, o, b, o],
        [b, o, o, o, o, o, o, o, o, o],
        [b, o, o, b, b, b, b, o, o, o],
        [b, o, o, o, o, o, o, o, o, o],
        [b, o, o, o, o, o, o, o, o, o],
        [b, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, b, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
    ];

    for (coordinate, player) in [
        (Coordinates { row: 1, col: 2 }, ALICE),
        (Coordinates { row: 4, col: 4 }, ALICE),
        (Coordinates { row: 2, col: 1 }, BOB),
    ] {
        let drop_bomb_result = Game::drop_bomb(state, coordinate, player);
        assert!(drop_bomb_result.is_ok());
    }

    state.change_game_phase(GamePhase::Play);

    for (index, side, player) in [
        (3, Side::North, ALICE),
        (7, Side::North, BOB),
        (2, Side::North, ALICE),
    ] {
        state = Game::drop_stone(state, player, side, index).unwrap();
    }

    // Alice destroyed her own stone by hiting a bomb on (1, 2)
    assert_eq!(
        state.get_player_score(&ALICE),
        Score::default() + NB_POINT_STONE * 2
    );
}

#[test]
fn should_play_a_game() {
    let o = Cell::Empty;
    let b = Cell::Block;

    let mut state = Game::new_game(ALICE, BOB, Some(INITIAL_SEED));
    state.board.cells = [
        [o, o, o, o, o, o, o, o, b, o],
        [b, o, o, o, o, o, o, o, o, o],
        [b, o, o, o, b, b, b, o, o, o],
        [b, o, o, o, o, o, o, o, o, o],
        [b, o, o, o, o, o, o, o, o, o],
        [b, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, b, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
    ];

    // players1 drops bombs
    let player1_num_bombs = state.get_player_bombs(&ALICE).unwrap();
    let drop_bomb_result = Game::drop_bomb(state, Coordinates { row: 0, col: 0 }, ALICE);
    assert!(drop_bomb_result.is_ok());
    state = drop_bomb_result.unwrap();
    assert_eq!(
        state.get_player_bombs(&ALICE).unwrap(),
        player1_num_bombs - 1
    );

    let drop_bomb_result = Game::drop_bomb(state, Coordinates { row: 0, col: 0 }, ALICE);
    assert!(
        drop_bomb_result.is_err(),
        "Player cannot drop two bombs in the same position"
    );
    assert_eq!(
        state.get_player_bombs(&ALICE).unwrap(),
        player1_num_bombs - 1
    );

    let drop_bomb_result = Game::drop_bomb(state, Coordinates { row: 9, col: 9 }, ALICE);
    assert!(drop_bomb_result.is_ok());
    state = drop_bomb_result.unwrap();
    assert_eq!(
        state.get_player_bombs(&ALICE).unwrap(),
        player1_num_bombs - 2
    );

    let drop_bomb_result = Game::drop_bomb(state, Coordinates { row: 7, col: 7 }, ALICE);
    assert!(drop_bomb_result.is_ok());
    state = drop_bomb_result.unwrap();
    assert_eq!(
        state.get_player_bombs(&ALICE).unwrap(),
        player1_num_bombs - 3
    );

    let drop_bomb_result = Game::drop_bomb(state, Coordinates { row: 6, col: 8 }, ALICE);
    assert!(drop_bomb_result.is_err());
    assert_eq!(
        drop_bomb_result.unwrap_err(),
        GameError::NoMoreBombsAvailable,
        "The player shouldn't have more bombs for dropping"
    );

    // players2 drops bombs
    let player2_num_bombs = state.get_player_bombs(&BOB).unwrap();
    let drop_bomb_result = Game::drop_bomb(state, Coordinates { row: 9, col: 0 }, BOB);
    assert!(drop_bomb_result.is_ok());
    state = drop_bomb_result.unwrap();
    assert_eq!(state.get_player_bombs(&BOB).unwrap(), player2_num_bombs - 1);

    let result = Game::drop_bomb(state, Coordinates { row: 9, col: 9 }, BOB);
    assert!(
        result.is_ok(),
        "A cell should hold bombs of different players"
    );
    state = result.unwrap();
    assert_eq!(state.get_player_bombs(&BOB).unwrap(), player2_num_bombs - 2);

    let result = Game::drop_bomb(state, Coordinates { row: 9, col: 3 }, BOB);
    assert!(result.is_ok());
    state = result.unwrap();
    assert_eq!(state.get_player_bombs(&BOB).unwrap(), player2_num_bombs - 3);

    assert_eq!(
        state.phase,
        GamePhase::Play,
        "The game should be in play phase after all bombs have been deployed"
    );

    let drop_stone_result = Game::drop_stone(state, BOB, Side::North, 0);
    assert!(drop_stone_result.is_err());
    assert_eq!(drop_stone_result.unwrap_err(), GameError::NotPlayerTurn);

    let drop_stone_result = Game::drop_stone(state, ALICE, Side::North, 0);
    assert!(drop_stone_result.is_ok());
    let mut state = drop_stone_result.unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 0, col: 0 }),
        Cell::Empty
    );

    state = Game::drop_stone(state, BOB, Side::North, 2).unwrap();
    state = Game::drop_stone(state, ALICE, Side::South, 8).unwrap();
    state = Game::drop_stone(state, BOB, Side::North, 2).unwrap();
    state = Game::drop_stone(state, ALICE, Side::South, 8).unwrap();
    state = Game::drop_stone(state, BOB, Side::North, 2).unwrap();

    // player 1 explodes bomb on 9,3 and player 2 loses stones on 9,2 and 8,2
    state = Game::drop_stone(state, ALICE, Side::North, 3).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 9, col: 2 }),
        Cell::Empty
    );
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 8, col: 2 }),
        Cell::Empty
    );

    // alice plays first square of stones
    state = Game::drop_stone(state, BOB, Side::North, 2).unwrap();
    state = Game::drop_stone(state, ALICE, Side::South, 8).unwrap();
    state = Game::drop_stone(state, BOB, Side::North, 2).unwrap();
    state = Game::drop_stone(state, ALICE, Side::South, 8).unwrap();
    state = Game::drop_stone(state, BOB, Side::North, 2).unwrap();
    state = Game::drop_stone(state, ALICE, Side::East, 1).unwrap();
    state = Game::drop_stone(state, BOB, Side::North, 2).unwrap();
    state = Game::drop_stone(state, ALICE, Side::East, 2).unwrap();

    // alice plays second square of stones
    state = Game::drop_stone(state, BOB, Side::East, 8).unwrap();
    state = Game::drop_stone(state, ALICE, Side::South, 9).unwrap();
    state = Game::drop_stone(state, BOB, Side::East, 8).unwrap();
    state = Game::drop_stone(state, ALICE, Side::South, 9).unwrap();

    // alice plays third square of stones
    state = Game::drop_stone(state, BOB, Side::East, 8).unwrap();
    state = Game::drop_stone(state, ALICE, Side::North, 5).unwrap();
    state = Game::drop_stone(state, BOB, Side::East, 8).unwrap();
    state = Game::drop_stone(state, ALICE, Side::North, 5).unwrap();
    state = Game::drop_stone(state, BOB, Side::East, 8).unwrap();
    state = Game::drop_stone(state, ALICE, Side::North, 6).unwrap();
    state = Game::drop_stone(state, BOB, Side::East, 8).unwrap();

    assert!(state.winner.is_none());
    let x = Cell::Stone(state.player_index(&ALICE));
    let y = Cell::Stone(state.player_index(&BOB));
    let p = Cell::Bomb([Some(state.player_index(&ALICE)), None]);
    let q = Cell::Bomb([Some(state.player_index(&BOB)), None]);
    assert_eq!(
        state.board.cells,
        [
            [o, o, o, o, o, x, o, o, b, o],
            [b, o, o, o, o, x, x, o, x, x],
            [b, o, o, o, b, b, b, o, x, x],
            [b, o, y, o, o, o, o, o, x, x],
            [b, o, y, o, o, o, o, o, x, o],
            [b, o, y, o, o, o, o, o, o, o],
            [o, o, y, o, o, o, b, o, o, o],
            [o, o, y, o, o, o, o, p, o, o],
            [y, y, y, y, y, y, o, o, o, o],
            [q, o, o, o, o, o, o, o, o, o],
        ]
    );

    // trigger winning condition and check winner
    state = Game::drop_stone(state, ALICE, Side::North, 6).unwrap();
    assert!(state.winner.is_some());
    assert_eq!(state.winner.unwrap(), ALICE);
}
