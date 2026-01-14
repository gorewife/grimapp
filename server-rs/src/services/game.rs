use crate::{database::Database, error::AppResult, models::{Game, GamePlayer, PlayerStats, ScriptStats}};
use sqlx::Row;

pub struct GameService {
    db: Database,
}

impl GameService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn get_game(&self, game_id: i32) -> AppResult<Option<Game>> {
        let game = sqlx::query_as::<_, Game>(
            "SELECT game_id, guild_id, script, custom_name, start_time, end_time, winner, 
                    player_count, players, is_active, created_at, completed_at, 
                    storyteller_id, category_id, storyteller_user_id 
             FROM games WHERE game_id = $1"
        )
        .bind(game_id)
        .fetch_optional(&self.db.pool)
        .await?;

        Ok(game)
    }

    pub async fn get_games(&self, limit: i64, offset: i64) -> AppResult<Vec<Game>> {
        let games = sqlx::query_as::<_, Game>(
            "SELECT game_id, guild_id, script, custom_name, start_time, end_time, winner, 
                    player_count, players, is_active, created_at, completed_at, 
                    storyteller_id, category_id, storyteller_user_id 
             FROM games 
             WHERE is_active = false 
             ORDER BY completed_at DESC NULLS LAST, created_at DESC 
             LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db.pool)
        .await?;

        Ok(games)
    }

    #[allow(dead_code)]
    pub async fn get_game_players(&self, game_id: i32) -> AppResult<Vec<GamePlayer>> {
        let players = sqlx::query_as::<_, GamePlayer>(
            "SELECT id, game_id, discord_id, player_name, seat_number, 
                    final_role_id, final_role_name, final_team, 
                    survived, winning_team, created_at,
                    starting_role_id, starting_role_name, starting_team 
             FROM game_players 
             WHERE game_id = $1 
             ORDER BY seat_number"
        )
        .bind(game_id)
        .fetch_all(&self.db.pool)
        .await?;

        Ok(players)
    }

    pub async fn count_total_games(&self) -> AppResult<i64> {
        let row = sqlx::query("SELECT COUNT(*) FROM games WHERE is_active = false")
            .fetch_one(&self.db.pool)
            .await?;

        Ok(row.get(0))
    }

    pub async fn count_total_players(&self) -> AppResult<i64> {
        let row = sqlx::query("SELECT COUNT(*) FROM game_players")
            .fetch_one(&self.db.pool)
            .await?;

        Ok(row.get(0))
    }

    pub async fn count_unique_players(&self) -> AppResult<i64> {
        let row = sqlx::query("SELECT COUNT(DISTINCT discord_id) FROM game_players WHERE discord_id IS NOT NULL")
            .fetch_one(&self.db.pool)
            .await?;

        Ok(row.get(0))
    }

    pub async fn count_active_games(&self) -> AppResult<i64> {
        let row = sqlx::query("SELECT COUNT(*) FROM games WHERE is_active = true")
            .fetch_one(&self.db.pool)
            .await?;

        Ok(row.get(0))
    }

    /// Get player statistics by Discord ID
    pub async fn get_player_stats(&self, discord_id: i64) -> AppResult<Option<PlayerStats>> {
        let stats = sqlx::query_as::<_, PlayerStats>(
            r#"
            SELECT 
                discord_id,
                MAX(player_name) as player_name,
                COUNT(*) as games_played,
                SUM(CASE WHEN winning_team THEN 1 ELSE 0 END) as wins,
                SUM(CASE WHEN NOT winning_team THEN 1 ELSE 0 END) as losses,
                ROUND(AVG(CASE WHEN survived THEN 1.0 ELSE 0.0 END) * 100, 2) as survival_rate,
                (
                    SELECT final_role_name 
                    FROM game_players 
                    WHERE discord_id = $1 AND final_role_name IS NOT NULL
                    GROUP BY final_role_name 
                    ORDER BY COUNT(*) DESC 
                    LIMIT 1
                ) as favorite_role
            FROM game_players
            WHERE discord_id = $1
            GROUP BY discord_id
            "#
        )
        .bind(discord_id)
        .fetch_optional(&self.db.pool)
        .await?;

        Ok(stats)
    }

    /// Get script statistics by script name
    pub async fn get_script_stats(&self, script_name: &str) -> AppResult<Option<ScriptStats>> {
        let stats = sqlx::query_as::<_, ScriptStats>(
            r#"
            SELECT 
                script as script_name,
                COUNT(*) as games_played,
                SUM(CASE WHEN winner = 'Good' THEN 1 ELSE 0 END) as good_wins,
                SUM(CASE WHEN winner = 'Evil' THEN 1 ELSE 0 END) as evil_wins,
                ROUND(AVG(player_count::numeric), 2) as average_player_count
            FROM games
            WHERE script = $1 AND is_active = false AND winner IS NOT NULL
            GROUP BY script
            "#
        )
        .bind(script_name)
        .fetch_optional(&self.db.pool)
        .await?;

        Ok(stats)
    }
}
