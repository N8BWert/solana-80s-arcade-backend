use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;
use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// AS OF October 13th 2022 at 3:20 PM the price of solana is $32.06
// That means that currently $0.25 is 0.00779788 SOL

const TWENTY_FIVE_CENTS: u64 = (0.0779766 * (LAMPORTS_PER_SOL as f32)) as u64;

#[program]
pub mod solana_80s_arcade_backend {
    use super::*;

    /// The function that is called to start a new arcade.
    /// 
    /// I choose not to be an overlord in this case and any person can decide that they do not want to be a part
    /// of the arcade and can start a new arcade.  I'm not 100% sure how they are going to do front-end for that arcade.
    /// (perhaps I will), however this is still necessary to start the original arcade (i.e. N8Cade <- working title hopefully I 
    /// come up with a better name.
    pub fn initialize_acrade(ctx: Context<InitArcade>) -> ProgramResult {
        // Get accounts from the context.
        let arcade_account = &mut ctx.accounts.arcade_account;
        let genesis_game_account = &mut ctx.accounts.genesis_game_account;
        let authority = &mut ctx.accounts.authority;

        // Set up the arcade state.
        arcade_account.authority = authority.key();
        arcade_account.most_recent_game_key = genesis_game_account.key();

        // If everything went well return Ok.
        Ok(())
    }

    /// This function should handel the creation of a new game/arcade machine.
    /// 
    /// Currently, I think that as games are added they will be added to the front part of the queue
    /// so users will see the newest games.  I don't really want a popularity contest so I'm thinking
    /// this is the best way to do this.
    /// 
    /// NOTE: I need to create a webgl build extension/add-on that creates a game wallet for games that 
    /// are to be added to the arcade.  Then I need to pass that wallet address into here to ensure the games
    /// get their money.
    pub fn create_game(
        ctx: Context<GamePost>,
        title: String,
        web_gl_hash: String,
        game_art_hash: String,
        game_wallet: Pubkey
    ) -> ProgramResult {
        // Get accounts from the context
        let game_account = &mut ctx.accounts.game_account;
        let arcade_account = &mut ctx.accounts.arcade_account;
        let owner = &mut ctx.accounts.owner;

        // Initialize game_account
        game_account.title = title;
        game_account.web_gl_hash = web_gl_hash;
        game_account.game_art_hash = game_art_hash;

        // TODO: figure out how to link this game_wallet and the web gl account wallet.
        game_account.game_wallet = game_wallet;
        game_account.owner_wallet = owner.key();
        game_account.later_game_key = arcade_account.most_recent_game_key;
        game_account.earlier_game_key = game_account.key();

        // Initialize leaderboard
        let first_place = Place {name: String::from("AAA"), wallet_key: game_wallet, score: 100};
        let second_place = Place {name: String::from("BBB"), wallet_key: game_wallet, score: 50};
        let third_place = Place {name: String::from("CCC"), wallet_key: game_wallet, score: 25};

        let leaderboard = Leaderboard {first_place, second_place, third_place};

        game_account.leaderboard = leaderboard;

        // Store most recent game key as current game key in arcade account.
        arcade_account.most_recent_game_key = game_account.key();

        // Emit the game created event.
        emit!(GameEvent {
            label: "CREATE".to_string(),
            game_id: game_account.key(),
            more_recent_game_id: None,
            less_recent_game_id: Some(game_account.later_game_key),
        });

        Ok(())
    }

    /// This function deletes a game, while making sure that the person deleting the machine/game is
    /// the person who owns it.  I would like anyone to upload whatever they want onto the arcade which
    /// may come back to bite me, but I think this is the best way to promote an open space.
    pub fn delete_game(ctx: Context<DeleteGame>) -> Result<()> {
        let game_account = &mut ctx.accounts.game_account;
        let earlier_game = &mut ctx.accounts.earlier_game;
        let later_game = &mut ctx.accounts.later_game;

        // Check that the signer is the same as the owner (I'm not sure that this is necessary,
        // but I don't want to leave this up to chance).
        if game_account.owner_wallet != ctx.accounts.owner_wallet.key() {
            return Err(Errors::CannotDeleteUnownedGame.into());
        }

        earlier_game.later_game_key = later_game.key();
        later_game.earlier_game_key = earlier_game.key();

        emit!(GameEvent {
            label: "DELETE".to_string(),
            game_id: game_account.key(),
            more_recent_game_id: Some(earlier_game.key()),
            less_recent_game_id: Some(later_game.key()),
        });

        Ok(())
    }

    /// This game deletes the most recent game.
    /// 
    /// I'm going to be completely honest, I think I'm going to delete this and instead make the arcade a 
    /// circular linked list, but this is here for now.
    pub fn delete_most_recent_game(ctx: Context<DeleteMostRecentGame>) -> Result<()> {
        let game_account = &mut ctx.accounts.game_account;
        let later_game = &mut ctx.accounts.later_game;

        // Check that the signer is the same as the owner (I'm not sure that this is necessary,
        // but I don't want to leave this up to chance).
        if game_account.owner_wallet != ctx.accounts.owner_wallet.key() {
            return Err(Errors::CannotDeleteUnownedGame.into());
        }

        // TODO: fix this, for now we're going to say a game without an earlier game has the earlier
        // game of itself.  I might make this that the front game refers to the last game to make a 
        // circular linked list, but I'm not sure.
        later_game.earlier_game_key = later_game.key();

        emit!(GameEvent {
            label: "DELETE".to_string(),
            game_id: game_account.key(),
            more_recent_game_id: None,
            less_recent_game_id: Some(later_game.key()),
        });

        Ok(())
    }

    /// This function delete s the oldest game.
    /// 
    /// I'm going to be honest, I think the circular linked list is better so this function will probably
    /// be scrapped.
    pub fn delete_oldest_game(ctx: Context<DeleteOldestGame>) -> Result<()> {
        let game_account = &mut ctx.accounts.game_account;
        let earlier_game = &mut ctx.accounts.earlier_game;

        // Check that the signer is the same as the owner (I'm not sure that this is necessary,
        // but I don't what to leave this up to chance).
        if game_account.owner_wallet != ctx.accounts.owner_wallet.key() {
            return Err(Errors::CannotDeleteUnownedGame.into());
        }

        // TODO: fix this, for now we're going to say that the oldest game will have no older game, 
        // however this might eventually be changed to make a circular linked list.
        earlier_game.later_game_key = earlier_game.key();

        emit!(GameEvent {
            label: "DELETE".to_string(),
            game_id: game_account.key(),
            more_recent_game_id: Some(earlier_game.key()),
            less_recent_game_id: None,
        });

        Ok(())
    }

    /// This function will be called by the webgl program to allow users to enter the game queue
    /// 
    /// Note: This code probably doesn't work so we'll need to test this a bit
    pub fn join_game_queue(ctx: Context<PlayGame>) -> ProgramResult {
        let game_account = &mut ctx.accounts.game_account;
        let payer = &mut ctx.accounts.payer;

        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &payer.key(),
            &game_account.game_wallet,
            TWENTY_FIVE_CENTS,
        );

        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                payer.to_account_info(),
                game_account.to_account_info(),
            ],
        )?;

        Ok(())
    }

    /// Whenever a game is played the game should make a call to the update leaderboard function to see if the leaderboard
    /// should be updated.
    pub fn update_leaderboard(ctx: Context<GameEnd>, player_name: String, score: u128, wallet_key: Pubkey) -> Result<()> {
        let game_account = &mut ctx.accounts.game_account;

        let name = match player_name.chars().count() {
            1 => player_name + "  ",
            2 => player_name + " ",
            3 => player_name,
            _ => return Err(Errors::IllegalName.into()),
        };

        let first = game_account.leaderboard.first_place.score;
        let second = game_account.leaderboard.second_place.score;
        let third = game_account.leaderboard.third_place.score;

        if score > third {
            let place = Place {name, wallet_key, score};

            if score > first {
                game_account.leaderboard.third_place = game_account.leaderboard.second_place.clone();
                game_account.leaderboard.second_place = game_account.leaderboard.first_place.clone();
                game_account.leaderboard.first_place = place;
            } else if score > second {
                game_account.leaderboard.third_place = game_account.leaderboard.second_place.clone();
                game_account.leaderboard.second_place = place;
            } else {
                game_account.leaderboard.third_place = place;
            }
        }

        Ok(())
    }
}

#[derive(Accounts)]
/// Context used to initialize the arcade.
pub struct InitArcade<'info> {
    #[account(init, payer = authority, space = 8 + 32 + 32)]
    // TODO: make sure there can only be one arcade state (MAYBE) -> it could be a feature to have multiple \_(**)_/
    pub arcade_account: Account<'info, ArcadeState>, // The accound for the arcade state (i.e. the pointer to the newest game).
    #[account(init, payer = authority, space = 8 + 940)]
    pub genesis_game_account: Account<'info, Game>, // The first game (i.e. the game that began the arcade).
    #[account(mut)]
    pub authority: Signer<'info>, // The person who pays for initializing the arcade (i.e. me).
    pub system_program: Program<'info, System>, // The system program to make sure the account created is associated with this program.
}

#[derive(Accounts)]
/// Context used to create a new game.
pub struct GamePost<'info> {
    #[account(init, payer = owner, space = 8 + 940)]
    pub game_account: Account<'info, Game>,
    #[account(mut)]
    pub arcade_account: Account<'info, ArcadeState>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DeleteGame<'info> {
    #[account(
        mut,
        has_one = owner_wallet,
        close = owner_wallet,
        constraint = game_account.key() == earlier_game.later_game_key,
        constraint = game_account.key() == later_game.earlier_game_key,
    )]
    pub game_account: Account<'info, Game>,
    #[account(mut)]
    pub earlier_game: Account<'info, Game>,
    #[account(mut)]
    pub later_game: Account<'info, Game>,
    pub owner_wallet: Signer<'info>,
}

#[derive(Accounts)]
pub struct DeleteMostRecentGame<'info> {
    #[account(
        mut,
        has_one = owner_wallet,
        close = owner_wallet,
        constraint = game_account.key() == later_game.earlier_game_key,
    )]
    pub game_account: Account<'info, Game>,
    #[account(mut)]
    pub later_game: Account<'info, Game>,
    pub owner_wallet: Signer<'info>,
}

#[derive(Accounts)]
pub struct DeleteOldestGame<'info> {
    #[account(
        mut,
        has_one = owner_wallet,
        close = owner_wallet,
        constraint = game_account.key() == earlier_game.later_game_key,
    )]
    pub game_account: Account<'info, Game>,
    #[account(mut)]
    pub earlier_game: Account<'info, Game>,
    pub owner_wallet: Signer<'info>,
}

#[derive(Accounts)]
pub struct PlayGame<'info> {
    #[account(mut)]
    pub game_account: Account<'info, Game>,
    #[account(mut)]
    pub payer: Signer<'info>,
}

#[derive(Accounts)]
pub struct GameEnd<'info> {
    #[account(mut)]
    pub game_account: Account<'info, Game>,
}

#[account]
/// The ArcadeState is the account that points to the most current game uploaded to the arcade.
/// 
/// I will probably end up paying rent for this to make sure it never dissappears, but it should honestly be incredibly cheap because the rent-exempt
/// minimum for this size is currently 0.00133632, which is about $0.044232.
/// 
/// size(ArcadeState) = size(Pubkey) + size(Pubkey) = 32 + 32 = 64 Bytes
pub struct ArcadeState {
    pub most_recent_game_key: Pubkey, // the key of the most recent game to be added to the arcade.
    pub authority: Pubkey, // the initializer of the arcade's key (aka my key).
}

#[account]
/// A game is the on-chain block that contains all important information about the game.
/// 
/// NOTE: All actual game data and game art will be stored on arweave to keep the gas prices down.  The only unitended consequence of this
/// is that games may not be modified after their upload, however we can delete a game if the person initializing the delete has the same
/// wallet public key as the owner_wallet.
/// 
/// size(Game) = 30*size(char) + 256 + 256 + size(Leaderboard) + 4*size(Pubkey) = 120 + 256 + 256 + 180 + 128 = 940 Bytes
pub struct Game {
    pub title: String, // A 30 character string for the name of the game.
    pub web_gl_hash: String, // The 256 Byte hash of the arweave location of the webGL build.
    pub game_art_hash: String, // The 256 Byte hash of the arweave location of the game art.
    pub leaderboard: Leaderboard, // The associated game leaderboard to rank players.
    pub earlier_game_key: Pubkey, // The key of the game posted one game after this game.
    pub later_game_key: Pubkey, // The key of the game posted one game before this game.
    pub owner_wallet: Pubkey, // The immutable wallet of the game owner (allows distribution of funds).
    pub game_wallet: Pubkey, // The wallet generated by teh unity webGL build to allow the game to hold the money for
                             // its rend and to hold money for distribution each month.
}

#[account]
/// The Leaderboard organizes the different player's places by score.
/// size(Leaderboard) = 3 * size(Pace) = 3 * 60 = 180 Bytes
pub struct Leaderboard {
    pub first_place: Place, // The person in first place.
    pub second_place: Place, // The person in second place.
    pub third_place: Place, // The person in third place.
}

#[account]
/// A place is a player's place on the leaderboard.
/// size(Place) = 3*size(char) + size(Pubkey) + size(u128) = 3*4 + 32 + 16 = 12 + 32 + 16 = 60 Bytes
pub struct Place {
    pub name: String, // 3 character string for traditional arcade scoreboard names.
    pub wallet_key: Pubkey, // public key of the placeholder to allow the transfer of funds.
    pub score: u128, // High score achieved by this individual.
}

#[event]
pub struct GameEvent {
    pub label: String, // label will be 'CREATE' and 'DELETE'.
    pub game_id: Pubkey, // created game.
    pub more_recent_game_id: Option<Pubkey>, // Useful for deleting games.
    pub less_recent_game_id: Option<Pubkey>, // Useful for creating games.
}

#[event]
pub struct LeaderboardEvent {
    pub player_name: String, // player_name will be the 3 character name chosen by the player.
    pub first_place_player_name: String, // The 3 character name of the first place player.
    pub second_place_player_name: String, // The 3 character name of the second place player.
    pub third_place_player_name: String, // The 3 character name of the third place player.
}

#[error_code]
pub enum Errors {
    #[msg("You cannot delete another user's games.  SHAME ON YOU")]
    CannotDeleteUnownedGame,

    #[msg("You cannot have a name that is more than 3 characters - or 0 characters")]
    IllegalName,
}