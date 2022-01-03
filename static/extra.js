function diff_year_month_day(dt1, dt2) 
{
    var time =(dt2.getTime() - dt1.getTime()) / 1000;
    var year  = Math.abs(Math.round((time/(60 * 60 * 24))/365.25));
    var month = Math.abs(Math.round(time/(60 * 60 * 24 * 7 * 4)));
    var days = Math.abs(Math.round(time/(3600 * 24)));
    return [year, month, days];//"Year :- " + year + " Month :- " + month + " Days :-" + days;
}


function getSince(updated_last_date)
{
    var diff_total = diff_year_month_day(new Date(), updated_last_date);
    var years = diff_total[0];
    var months = diff_total[1];
    var days = diff_total[2];

    var final = 0;
    var datetype = "x";
    if(years>0 && months > 12){
        if(years>1){
            datetype = "years"
        } else {
            datetype = "year"
        }
        final = years;
    } else if (months > 0){
        if(months>1){
            datetype = "months"
        } else {
            datetype = "month"
        }
        final = months;
    } else if (days > 0){
        if(days>1){
            datetype = "days"
        } else {
            datetype = "day"
        }
        final = days;
    }
    return final + " " + datetype + " ago";
}