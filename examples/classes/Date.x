function isLeapYear(year) {
    if (year % 4 != 0) {
        return false;
    }
    if (year % 100 != 0) {
        return true;
    }
    if (year % 400 != 0) {
        return false;
    }
    return true;
}

function daysInMonth(month, year) {
    let days = [
        31, // Jan
        28, // Feb
        31, // Mar
        30, // Apr
        31, // May
        30, // Jun
        31, // Jul
        31, // Aug
        30, // Sep
        31, // Oct
        30, // Nov
        31, // Dec
    ];

    if (month == 1) {
        if (isLeapYear(year)) {
            return 29;
        }
    }

    return days[month];
}

function floorDiv(a, b) {
    let r = a / b;
    let f = 0;
    if (r >= 0) {
        f = r - (r % 1);
    } else {
        // Para números negativos, arredonda para baixo
        let mod = r % 1;
        if (mod === 0) {
            f = r;
        } else {
            f = r - mod - 1;
        }
    }
    return f;
}

function getDayOfMonth(timestamp) {
    let msPerDay = 1000 * 60 * 60 * 24;
    let totalDays = floorDiv(timestamp, msPerDay);

    let year = 1970;
    for (let i = 0; true; i = i + 1) {
        let daysThisYear = 365;
        if (isLeapYear(year)) {
            daysThisYear = 366;
        }
        if (totalDays >= daysThisYear) {
            totalDays = totalDays - daysThisYear;
            year = year + 1;
        } else {
            break;
        }
    }

    let month = 0;
    for (let i = 0; true; i = i + 1) {
        let dim = daysInMonth(month, year);
        if (totalDays >= dim) {
            totalDays = totalDays - dim;
            month = month + 1;
        } else {
            break;
        }
    }

    return totalDays + 1;
}

function getMonth(timestamp) {
    let msPerDay = 1000 * 60 * 60 * 24;
    let totalDays = floorDiv(timestamp, msPerDay);

    let year = 1970;
    for (let i = 0; true; i = i + 1) {
        let daysThisYear = 365;
        if (isLeapYear(year)) {
            daysThisYear = 366;
        }
        if (totalDays >= daysThisYear) {
            totalDays = totalDays - daysThisYear;
            year = year + 1;
        } else {
            break;
        }
    }

    let month = 0;
    for (let i = 0; true; i = i + 1) {
        let dim = daysInMonth(month, year);
        if (totalDays >= dim) {
            totalDays = totalDays - dim;
            month = month + 1;
        } else {
            break;
        }
    }
    month = month + 1;

    let formatedMonth = "";
    if (month < 10) {
        formatedMonth = "0" + month;
    } else {
        formatedMonth = "" + month;
    }
    return formatedMonth;
}

function getYear(timestamp) {
    let msPerDay = 1000 * 60 * 60 * 24;
    let totalDays = floorDiv(timestamp, msPerDay);

    let year = 1970;
    for (let i = 0; true; i = i + 1) {
        let daysThisYear = 365;
        if (isLeapYear(year)) {
            daysThisYear = 366;
        }
        if (totalDays >= daysThisYear) {
            totalDays = totalDays - daysThisYear;
            year = year + 1;
        } else {
            break;
        }
    }

    return year;
}
export class Date {
    time = now();
    constructor(time) {
        if (time != null) {
            this.time = time;
        }
    }
    static now() {
        return now();
    }
    getDayOfMonth() {
        return getDayOfMonth(this.time);
    }
    getMonth() {
        return getMonth(this.time);
    }
    getYear() {
        return getYear(this.time);
    }

    toString() {
        return this.getDayOfMonth() + "/" + this.getMonth() + "/" + this.getYear();
    }
}
