use std::convert::TryInto;
pub fn get_datetime(epoch_year: u32, duration_sec: u64) -> (u32,u32,u32, u32,u32,u32,u8) { // year, month, day, hour, minute, second, a day in a week after epoch day
    let mut mon_days = [31,
	28,
	31,
	30,
	31,
	30,
	31,
	31,
	30,
	31,
	30,
	31];

    let mut days : u32 = (duration_sec / 86400) as u32;
    let sec_in_day = (duration_sec % 86400) as u32;
    let mins_in_day = sec_in_day / 60;
    let sec_in_min = sec_in_day % 60;
    let hour_in_day = mins_in_day / 60;
	let min_in_hour = mins_in_day % 60;
	let mut curr_year = epoch_year;
	let remaining_days_week : u8 = (days % 7).try_into().unwrap();
	
	if days > year_len(curr_year) {
		loop {
			days -= year_len(curr_year);
			curr_year += 1;
			if days < year_len(curr_year) {
				break ;
			}
		}
	}
	if year_len(curr_year) == 366 {
		mon_days[1] = 29;
	}
	let mut current_month: u32 = 0;
	if days > 0 {
		loop {
			if days < mon_days[current_month as usize] {
				break;
			}
			days -= mon_days[current_month as usize];
			current_month += 1;
		}
	}
	(curr_year, current_month+1, days+1, hour_in_day, min_in_hour, sec_in_min,remaining_days_week)
}

#[inline]
fn year_len(year:u32) -> u32 {
	if (year%4) == 0 && (year%100) != 0 || (year%400) == 0 {
		366
	} else {
		365
	}
}