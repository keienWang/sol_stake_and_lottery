pub const SECONDS_IN_MINUTE: u64 = 60;
pub const SECONDS_IN_HOUR: u64 = 60 * SECONDS_IN_MINUTE;
pub const SECONDS_IN_DAY: u64 = 24 * SECONDS_IN_HOUR;
pub const SECONDS_IN_YEAR: u64 = 365 * SECONDS_IN_DAY;
pub const SECONDS_IN_LEAP_YEAR: u64 = 366 * SECONDS_IN_DAY;

pub const DAYS_IN_MONTH: [u64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
pub const DAYS_IN_MONTH_LEAP_YEAR: [u64; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const UTC_OFFSET_SECONDS: u64 = 8 * 60 * 60; // UTC+8 (东八区)

fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

//参数为utc+0时间，得出结果为utc+8 日期
pub fn timestamp_to_ymd(mut unix_timestamp: u64) -> (u64, u64, u64) {
    let mut timestamp = unix_timestamp;

    let mut year = 1970;

    timestamp += UTC_OFFSET_SECONDS;
    // 减去年份的秒数，直到找到具体年份
    while timestamp >= 0 {
        let year_seconds = if is_leap_year(year) {
            SECONDS_IN_LEAP_YEAR
        } else {
            SECONDS_IN_YEAR
        };

        if timestamp >= year_seconds {
            timestamp -= year_seconds;
            year += 1;
        } else {
            break;
        }
    }

    // 找到月份
    let mut month = 0;
    let days_in_month = if is_leap_year(year) {
        &DAYS_IN_MONTH_LEAP_YEAR
    } else {
        &DAYS_IN_MONTH
    };

    while timestamp >= 0 {
        let month_days = days_in_month[month] * SECONDS_IN_DAY;
        if timestamp >= month_days {
            timestamp -= month_days;
            month += 1;
        } else {
            break;
        }
    }

    let day = (timestamp / SECONDS_IN_DAY) + 1;

    (year, (month + 1) as u64, day) // 返回年月日
}

// 日期转换为 Unix 时间戳， 参数为utc+8 时间，得出utc+0 时间戳
fn date_to_timestamp(year: u64, month: u64, day: u64) -> u64 {
    // const SECONDS_IN_DAY: u64 = 24 * 60 * 60;
    const DAYS_IN_MONTH: [u64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    let mut timestamp = 0;

    // 加上前面的年数
    for y in 1970..year {
        timestamp += if is_leap_year(y) {
            SECONDS_IN_DAY * 366
        } else {
            SECONDS_IN_DAY * 365
        };
    }

    // 加上前面的月数
    for m in 1..month {
        timestamp += SECONDS_IN_DAY * if m == 2 && is_leap_year(year) {
            29
        } else {
            DAYS_IN_MONTH[(m - 1) as usize]
        };
    }

    // 加上天数
    timestamp += SECONDS_IN_DAY * (day - 1);
    timestamp -= UTC_OFFSET_SECONDS;

    timestamp
}

/// 根据质押类型，计算出时间戳，stake_type 为 0 代表三个月，1 代表六个月，2 代表十二个月，
pub fn generate_release_timestamps(purchase_timestamp: u64, stake_type: u64) -> u64 {
    let (mut year, mut month, mut day) = timestamp_to_ymd(purchase_timestamp);
    let addtime = date_to_timestamp(year, month, day);
    let add = purchase_timestamp - addtime;
    // 根据质押类型设置需要加的月份数
    let months_to_add = match stake_type {
        0 => 3,        // 三个月
        1 => 6,        // 六个月
        2 => 12,       // 十二个月
        _ => return 0, // 无效的 stake_type，返回 0
    };

    // 计算新日期
    for _ in 0..months_to_add {
        // 更新到下一个月
        if month == 12 {
            month = 1;
            year += 1;
        } else {
            month += 1;
        }
    }

    // 返回计算后的时间戳
    date_to_timestamp(year, month, day) + add
}

/// 根据质押类型，计算出时间戳，stake_type 为 0 代表30分钟，1代表60分钟，2代表120分钟
pub fn test_generate_release_timestamp(purchase_timestamp: u64, stake_type: u64) -> u64 {
    let (mut year, mut month, mut day) = timestamp_to_ymd(purchase_timestamp);

    // 根据质押类型设置需要加的分钟数
    let minutes_to_add = match stake_type {
        0 => 30,       // 30分钟
        1 => 60,       // 60分钟
        2 => 120,      // 120分钟
        _ => return 0, // 无效的 stake_type，返回 0
    };

    // 将分钟转换为秒数
    let seconds_to_add = minutes_to_add * 60;

    // 增加秒数（转换为时间戳）
    purchase_timestamp + seconds_to_add
}
