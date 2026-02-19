// Constants to use in the program
pub const SECONDS_IN_DAY: i64 = 86400;                      // 24 hours
pub const SECONDS_IN_HOUR: i64 = 3600;                      // 1 hour
pub const SECONDS_IN_MINUTE: i64 = 60;                      // 1 minute
pub const OPEN_HOUR: i64 = 9 * SECONDS_IN_HOUR;             // 9am UTC
pub const CLOSE_HOUR: i64 = 17 * SECONDS_IN_HOUR;           // 5pm UTC
pub const REWARDS_TOLERANCE: i64 = 15 * SECONDS_IN_MINUTE;  // 15 minutes
pub const REWARDS_LAMPORTS: u64 = 10000000;                 // 0.001 SOL