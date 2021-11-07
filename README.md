# today

## Commands

### The `TASK` command

```
TASK title
(DATE annotation)*
[FROM annotation]
[UNTIL annotation]
(DONE annotation)*
    long description
```

### The `NOTE` command

```
NOTE title
(DATE annotation)+
[FROM annotation]
[UNTIL annotation]
(DONE annotation)*
    long description
```

### The `BIRTHDAY` command

```
BIRTHDAY name
BDATE annotation
```

## Annotations

### The `DATE` annotation

Most commands allow or require `DATE` annotations. They are roughly structured
like `DATE start [-- end]`. The `end` part can only contain time-related
information if the `start` specifies a time.

More specifically, there are three variants of the `DATE` annotation:
```
DATE date [delta] [time] [-- [date] [delta] [time]] [; delta]
DATE weekday [time] [-- [weekday] [delta] [time]]
DATE formula [delta] [time] [-- [delta] [time]]
```

In all three cases, the `end` must contain at least one of the optional elements
if it is present. Deltas in the `end` may represent fractional days (e. g.
`+3h`) as long as they are not immediately followed by a time and the `start`
includes a time. Other deltas may only represent whole-day intervals (they may
contain sub-day specifiers like `+24h` or `+25h-60m` as long as they sum to a
whole-day interval).

In the case of the `date` variant, a repetition delta can be specified following
a semicolon.

If multiple `DATE` annotations from a single command start on the same date, all
except the first are ignored.

### The `FROM` and `UNTIL` annotations

Commands that allow `DATE` annotations also allow the `FROM` and `UNTIL`
annotations. These can be used to restrict occurrences of entries.

The `FROM` annotation has the form `FROM date` and only keeps occurences that
end at or after the specified date. These occurences may begin before the
specified date.

The `UNTIL` annotation has the form `UNTIL date` and only keeps occurrences that
start at or before the specified date. These occurrences may extend past the
specified date.

### The `BDATE` annotations

This is a very simple date annotation for birthdays. It has the form
`BDATE date` where the year may optionally be replaced by a `?`.

If the year is specified, the person's current age is displayed in the birthday
occurrence title.

## Examples
```
NOTE Spielerunde
DATE sun 22:00 -- 24:00
DATE sun 22:00 -- 00:00
DATE sun 22:00 -- +2h
DATE (wd = sun) 22:00 -- 24:00
DATE 2021-11-07 22:00 -- 24:00; +w
DATE 2021-11-07 22:00 -- +2h; +w

NOTE daily
DATE *
DATE (true)
DATE 2021-11-07; +d

NOTE on weekends
DATE (wd = sat | wd = sun)

NOTE weekends
DATE sat -- sun
DATE 2021-11-06 -- 2021-11-07; +w
DATE 2021-11-06 -- +d; +w
DATE (wd = sat) -- +d

NOTE last of each month
DATE (m = 1) -d

BIRTHDAY Max
BDATE 1987-05-14

BIRTHDAY Martha
BDATE ?-09-21
```
