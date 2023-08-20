#!/usr/bin/env python3
#
# Copyright (c) 2023 Robert Escriva


import argparse
import base64
import collections
import datetime
import heapq
import math
import sqlite3
import unittest
import uuid
import zoneinfo


ONE_DAY = datetime.timedelta(days=1)


Slider = collections.namedtuple('Slider', ('before', 'after'))
Latest = collections.namedtuple('Latest', ('when', 'what'))


# A rhythm that must be done each day.  Daily processes can only be canceled; they cannot rescheduled because every
# other day has a Daily already.
class Daily:

    SKIP_BEAT_WITHIN_SLIDER = False

    def __init__(self, id, desc):
        self.id = id
        self.desc = desc

    def start_beat(self, date):
        return date

    def next_beat(self, date):
        return date + ONE_DAY

    def prev_beat(self, date):
        return date - ONE_DAY

    @property
    def slider(self):
        return Slider(0, 0)

    def approximate_periodicity(self):
        return 1

    def as_dict(self):
        return {'rhythm_id': self.id, 'desc': self.desc}

    def __str__(self):
        return self.id + ' ' + self.desc


# A rhythm that must be done once per month, on a particular day of the month.
class Monthly:

    SKIP_BEAT_WITHIN_SLIDER = True

    def __init__(self, id, desc, dotm, slider=None):
        self.id = id
        self.desc = desc
        self.dotm = dotm
        self._slider = slider or Slider(0, 0)

    def start_beat(self, date):
        # NOTE(rescrv): Intentionally bail if on this day of the month.
        while date.day != self.dotm:
            date = date + ONE_DAY
        return date

    def next_beat(self, date):
        if date.day == self.dotm:
            date = date + ONE_DAY
        while date.day != self.dotm:
            date = date + ONE_DAY
        return date

    def prev_beat(self, date):
        if date.day == self.dotm:
            date = date - ONE_DAY
        while date.day != self.dotm:
            date = date - ONE_DAY
        return date

    @property
    def slider(self):
        return self._slider

    def approximate_periodicity(self):
        return 31

    def as_dict(self):
        return {'rhythm_id': self.id, 'desc': self.desc, 'dotm': self.dotm,
                'slider_before': self.slider.before,
                'slider_after': self.slider.after}

    def __str__(self):
        return '{} dotm:{} slider:{},{} {}'.format(self.id, self.dotm, self.slider.before, self.slider.after, self.desc)


# A rhythm that should be done on a particular day of the week.
class WeekDaily:

    SKIP_BEAT_WITHIN_SLIDER = True

    def __init__(self, id, desc, dotw, slider=None):
        self.id = id
        self.desc = desc
        self.dotw = dotw
        self._slider = slider or Slider(0, 0)

    def start_beat(self, date):
        # NOTE(rescrv): Intentionally bail if on this day of the week.
        while date.weekday() != self.dotw:
            date = date + ONE_DAY
        return date

    def next_beat(self, date):
        if date.weekday() == self.dotw:
            date = date + ONE_DAY
        while date.weekday() != self.dotw:
            date = date + ONE_DAY
        return date

    def prev_beat(self, date):
        if date.weekday() == self.dotw:
            date = date - ONE_DAY
        while date.weekday() != self.dotw:
            date = date - ONE_DAY
        return date

    @property
    def slider(self):
        return self._slider

    def approximate_periodicity(self):
        return 7

    def as_dict(self):
        return {'rhythm_id': self.id, 'desc': self.desc, 'dotw': self.dotw,
                'slider_before': self.slider.before,
                'slider_after': self.slider.after}

    def __str__(self):
        return '{} dotw:{} slider:{},{} {}'.format(self.id, self.dotw, self.slider.before, self.slider.after, self.desc)


# A flexible rhythm that recurs at approximately every N days.  The scheduling system takes into account the N value and
# decides the scheduling of the rhythm based upon history.
class EveryNDays:

    SKIP_BEAT_WITHIN_SLIDER = False

    def __init__(self, id, desc, n, slider=None):
        self.id = id
        self.desc = desc
        self.n = n
        self._slider = slider or Slider(0, 0)

    def start_beat(self, start):
        if self.n >= 7:
            return start
        date = start - 7 * ONE_DAY
        while date.weekday() != self.n:
            date = date + ONE_DAY
        while date < start:
            date = self.next_beat(date)
        return date

    def next_beat(self, date):
        return date + datetime.timedelta(days=self.n)

    def prev_beat(self, date):
        return date - datetime.timedelta(days=self.n)

    @property
    def slider(self):
        return self._slider

    def approximate_periodicity(self):
        return self.n

    def as_dict(self):
        return {'rhythm_id': self.id, 'desc': self.desc, 'n': self.n,
                'slider_before': self.slider.before,
                'slider_after': self.slider.after}

    def __str__(self):
        return '{} n:{} slider:{},{} {}'.format(self.id, self.n, self.slider.before, self.slider.after, self.desc)


def continuing_beat(rhythm, start, last_seen):
    if last_seen is None:
        return rhythm.start_beat(start)
    # This beat should necessarily the first beat after last_seen.
    beat = rhythm.next_beat(last_seen)
    # Somethimes slider will move to before a given date, e.g. a Thursday task move to Wednesday.
    # skip_beat_within_slider == true says that we shouldn't take the beat when it's within slider.before of the last
    # seen.  If it is, advance.
    if rhythm.SKIP_BEAT_WITHIN_SLIDER and (beat - last_seen).days < rhythm.slider.before:
        beat = rhythm.next_beat(beat)
    while beat < start:
        beat = rhythm.next_beat(beat)
    return beat


def create_table_if_not_exist(cursor, table_name, table_definition):
    query = "SELECT name FROM sqlite_master WHERE type='table' AND name=?"
    if len(list(cursor.execute(query, (table_name,)))) == 0:
        cursor.execute(table_definition)


def generate_id():
    while True:
        x = base64.urlsafe_b64encode(uuid.uuid4().bytes).rstrip(b'=').decode('ascii')
        if not x.startswith('-'):
            return x


def parse_sql_date_or_datetime_as_date(when):
    if isinstance(when, datetime.datetime):
        return when.date()
    elif isinstance(when, datetime.date):
        return when
    PATTERNS = [
        '%Y-%m-%d %H:%M:%S.%f',
        '%Y-%m-%d %H:%M:%S',
        '%Y-%m-%d',
    ]
    err = None
    for pattern in PATTERNS:
        try:
            return datetime.datetime.strptime(when, pattern).date()
        except ValueError as e:
            err = e
    if err is not None:
        raise err


def mean(X):
    total = 0
    count = 0
    for x in X:
        count += x
        total += 1
    if total == 0:
        return 0
    return count / total


def schedule_heap_push(smooth_rhythms, smoothing, rhythm, next_day):
    num_choices = len(smoothing.remaining_choices)
    heapq.heappush(smooth_rhythms, (next_day, num_choices, rhythm.approximate_periodicity(), rhythm.id, smoothing, rhythm))


class Smoothing:

    def __init__(self, rhythm, original_beat, start, limit):
        self.rhythm = rhythm
        self.original_beat = original_beat
        self.remaining_choices = []
        self.passed_over_choices = []
        for x in self.all_options():
            if x >= start and x < limit:
                self.remaining_choices.append(x)

    def all_options(self):
        yield self.original_beat
        for i in range(1, self.rhythm.slider.before + 1):
            yield self.original_beat - i * ONE_DAY
        for i in range(1, self.rhythm.slider.after + 1):
            yield self.original_beat + i * ONE_DAY

    def shift_one(self):
        assert self.remaining_choices
        picked = self.remaining_choices[0]
        self.passed_over_choices.append(picked)
        self.remaining_choices = self.remaining_choices[1:]
        return picked

class CadenceApp:

    def __init__(self, conn, email, tz=None, today=None):
        self._conn = conn
        self._email = email
        self._tz = tz
        self._today = today
        create_table_if_not_exist(self._conn, 'rhythms',
                '''CREATE TABLE rhythms (
                email text NOT NULL,
                id text NOT NULL,
                desc text NOT NULL,
                PRIMARY KEY (email, id)
                )''')
        create_table_if_not_exist(self._conn, 'dailies',
                '''CREATE TABLE dailies (
                email text NOT NULL,
                id text NOT NULL,
                PRIMARY KEY (email, id)
                )''')
        create_table_if_not_exist(self._conn, 'monthlies',
                '''CREATE TABLE monthlies (
                email text NOT NULL,
                id text NOT NULL,
                dotm integer NOT NULL,
                slider_before integer NOT NULL,
                slider_after integer NOT NULL,
                PRIMARY KEY (email, id)
                )''')
        create_table_if_not_exist(self._conn, 'week_dailies',
                '''CREATE TABLE week_dailies (
                email text NOT NULL,
                id text NOT NULL,
                dotw integer NOT NULL,
                slider_before integer NOT NULL,
                slider_after integer NOT NULL,
                PRIMARY KEY (email, id)
                )''')
        create_table_if_not_exist(self._conn, 'every_n_days',
                '''CREATE TABLE every_n_days (
                email text NOT NULL,
                id text NOT NULL,
                n integer NOT NULL,
                slider_before integer NOT NULL,
                slider_after integer NOT NULL,
                PRIMARY KEY (email, id)
                )''')
        create_table_if_not_exist(self._conn, 'events',
                '''CREATE TABLE events (
                email text NOT NULL,
                id text NOT NULL,
                what text NOT NULL,
                dt date NOT NULL,
                PRIMARY KEY (email, id, dt)
                )''')
        create_table_if_not_exist(self._conn, 'spoons',
                '''CREATE TABLE spoons (
                email text NOT NULL,
                dt date NOT NULL,
                spoons integer NOT NULL DEFAULT 0,
                PRIMARY KEY (email, dt)
                )''')

    def today(self):
        if self._today is not None:
            return self._today
        utcnow = datetime.datetime.now(tz=zoneinfo.ZoneInfo('UTC'))
        if self._tz in zoneinfo.available_timezones():
            return utcnow.astimezone(zoneinfo.ZoneInfo(self._tz)).date()
        return utcnow.date()

    def add_daily(self, desc):
        id = generate_id()
        self._conn.execute('INSERT INTO rhythms (email, id, desc) VALUES (?, ?, ?)', (self._email, id, desc))
        self._conn.execute('INSERT INTO dailies (email, id) VALUES (?, ?)', (self._email, id))
        self._conn.commit()
        return id

    def add_monthly(self, desc, dotm, slider_before=0, slider_after=0):
        dotm = int(dotm)
        if dotm < 1 or dotm > 31:
            raise ValueError('day of the month must be [1, 31]')
        id = generate_id()
        self._conn.execute('INSERT INTO rhythms (email, id, desc) VALUES (?, ?, ?)', (self._email, id, desc))
        self._conn.execute('INSERT INTO monthlies (email, id, dotm, slider_before, slider_after) VALUES (?, ?, ?, ?, ?)',
                           (self._email, id, dotm, slider_before, slider_after))
        self._conn.commit()
        return id

    def add_week_daily(self, desc, dotw, slider_before=0, slider_after=0):
        dotw = int(dotw)
        if dotw < 0 or dotw > 6:
            raise ValueError('day of the week must be [0, 6]')
        id = generate_id()
        self._conn.execute('INSERT INTO rhythms (email, id, desc) VALUES (?, ?, ?)', (self._email, id, desc))
        self._conn.execute('INSERT INTO week_dailies (email, id, dotw, slider_before, slider_after) VALUES (?, ?, ?, ?, ?)',
                           (self._email, id, dotw, slider_before, slider_after))
        self._conn.commit()
        return id

    def add_every_n_days(self, desc, n, slider_before=0, slider_after=0):
        n = int(n)
        if n < 2:
            raise ValueError('n cannot be smaller than 2')
        if n > 90:
            raise ValueError('n cannot be greater than 90')
        id = generate_id()
        self._conn.execute('INSERT INTO rhythms (email, id, desc) VALUES (?, ?, ?)', (self._email, id, desc))
        self._conn.execute('INSERT INTO every_n_days (email, id, n, slider_before, slider_after) VALUES (?, ?, ?, ?, ?)',
                           (self._email, id, n, slider_before, slider_after))
        self._conn.commit()
        return id

    def _edit_desc(self, rhythm_id, desc):
        assert desc is not None
        self._conn.execute('UPDATE rhythms SET desc = ? WHERE email = ? AND id = ?', (desc, self._email, rhythm_id))

    def _edit_sliders(self, rhythm_id, table, slider_before=None, slider_after=None):
        if slider_before:
            self._conn.execute('UPDATE {} SET slider_before = ? WHERE email = ? AND id = ?'.format(table), (slider_before, self._email, rhythm_id))
        if slider_after:
            self._conn.execute('UPDATE {} SET slider_after = ? WHERE email = ? AND id = ?'.format(table), (slider_after, self._email, rhythm_id))

    def edit_daily(self, rhythm_id, desc=None):
        if desc is not None:
            self._edit_desc(rhythm_id, desc)
        self._conn.commit()

    def edit_monthly(self, rhythm_id, desc=None, dotm=None, slider_before=None, slider_after=None):
        if dotm is not None:
            dotm = int(dotm)
            if (dotm < 1 or dotm > 31):
                raise ValueError('day of the month must be [1, 31]')
        if desc is not None:
            self._edit_desc(rhythm_id, desc)
        if dotm is not None:
            self._conn.execute('UPDATE monthlies SET dotm = ? WHERE email = ? AND id = ?', (dotm, self._email, rhythm_id))
        if slider_before or slider_after:
            self._edit_sliders(rhythm_id, 'monthlies', slider_before=slider_before, slider_after=slider_after)
        self._conn.commit()

    def edit_week_daily(self, rhythm_id, desc=None, dotw=None, slider_before=None, slider_after=None):
        if dotw is not None:
            dotw = int(dotw)
            if (dotw < 0 or dotw > 6):
                raise ValueError('day of the week must be [0, 6]')
        if desc is not None:
            self._edit_desc(rhythm_id, desc)
        if dotw is not None:
            self._conn.execute('UPDATE week_dailies SET dotw = ? WHERE email = ? AND id = ?', (dotw, self._email, rhythm_id))
        if slider_before or slider_after:
            self._edit_sliders(rhythm_id, 'week_dailies', slider_before=slider_before, slider_after=slider_after)
        self._conn.commit()

    def edit_every_n_days(self, rhythm_id, desc=None, n=None, slider_before=None, slider_after=None):
        if n is not None:
            n = int(n)
            if n < 2:
                raise ValueError('n cannot be smaller than 2')
            if n > 90:
                raise ValueError('n cannot be greater than 90')
        if desc is not None:
            self._edit_desc(rhythm_id, desc)
        if n is not None:
            self._conn.execute('UPDATE every_n_days SET n = ? WHERE email = ? AND id = ?', (n, self._email, rhythm_id))
        if slider_before or slider_after:
            self._edit_sliders(rhythm_id, 'every_n_days', slider_before=slider_before, slider_after=slider_after)
        self._conn.commit()

    def delete_rhythm(self, rhythm_id):
        self._conn.execute('DELETE FROM rhythms WHERE email = ? AND id = ?', (self._email, rhythm_id))
        self._conn.execute('DELETE FROM dailies WHERE email = ? AND id = ?', (self._email, rhythm_id))
        self._conn.execute('DELETE FROM monthlies WHERE email = ? AND id = ?', (self._email, rhythm_id))
        self._conn.execute('DELETE FROM week_dailies WHERE email = ? AND id = ?', (self._email, rhythm_id))
        self._conn.execute('DELETE FROM every_n_days WHERE email = ? AND id = ?', (self._email, rhythm_id))
        self._conn.commit()

    def list_rhythms(self):
        for daily in self.list_dailies():
            yield daily
        for monthly in self.list_monthlies():
            yield monthly
        for week_daily in self.list_week_dailies():
            yield week_daily
        for every_n in self.list_every_n_days():
            yield every_n

    def list_dailies(self):
        for row in self._conn.execute('''
            SELECT rhythms.id, desc
            FROM rhythms, dailies
            WHERE rhythms.email=?
                AND rhythms.email = dailies.email
                AND rhythms.id = dailies.id
                ''', (self._email,)):
            yield Daily(row[0], row[1])

    def list_monthlies(self):
        for row in self._conn.execute('''
            SELECT rhythms.id, desc, dotm, slider_before, slider_after
            FROM rhythms, monthlies
            WHERE rhythms.email=?
                AND rhythms.email = monthlies.email
                AND rhythms.id = monthlies.id
                ''', (self._email,)):
            yield Monthly(row[0], row[1], row[2], Slider(row[3], row[4]))

    def list_week_dailies(self):
        for row in self._conn.execute('''
            SELECT rhythms.id, desc, dotw, slider_before, slider_after
            FROM rhythms, week_dailies
            WHERE rhythms.email=?
                AND rhythms.email = week_dailies.email
                AND rhythms.id = week_dailies.id
                ''', (self._email,)):
            yield WeekDaily(row[0], row[1], row[2], Slider(row[3], row[4]))

    def list_every_n_days(self):
        for row in self._conn.execute('''
            SELECT rhythms.id, desc, n, slider_before, slider_after
            FROM rhythms, every_n_days
            WHERE rhythms.email=?
                AND rhythms.email = every_n_days.email
                AND rhythms.id = every_n_days.id
                ''', (self._email,)):
            yield EveryNDays(row[0], row[1], row[2], Slider(row[3], row[4]))

    def done(self, id, when=None):
        when = when or self.today()
        self._conn.execute('INSERT INTO events (email, id, what, dt) VALUES (?, ?, "done", ?) ON CONFLICT DO NOTHING', (self._email, id, when))
        self._conn.commit()

    def last_done(self, rhythm_id):
        for row in self._conn.execute('''
            SELECT dt
            FROM events
            WHERE email=? AND id=?
                AND what="done"
                ''', (self._email, rhythm_id)):
            return parse_sql_date_or_datetime_as_date(row[0])
        return None

    def defer(self, id, when=None):
        when = when or self.today()
        self._conn.execute('INSERT INTO events (email, id, what, dt) VALUES (?, ?, "defer", ?) ON CONFLICT DO NOTHING', (self._email, id, when))
        self._conn.commit()

    def schedule(self, start=None, limit=None):
        return self._schedule(start, limit, {})

    def _schedule(self, start, limit, watermarks):
        # Our output
        schedule = {}
        # Gather events.
        events = collections.defaultdict(list)
        for event in self._conn.execute('SELECT id, what, dt FROM events WHERE email=?', (self._email,)):
            when = parse_sql_date_or_datetime_as_date(event[2])
            what = event[1]
            events[event[0]].append(Latest(when=when, what=what))
        events = dict(events)
        for event_list in events.values():
            event_list.sort()
        # Figure out how many spoons are available.
        spoons = {}
        for spoon in self._conn.execute('SELECT dt, spoons FROM spoons WHERE email=?', (self._email,)):
            when = parse_sql_date_or_datetime_as_date(spoon[0])
            spoons[when] = spoon[1]
        # Create a time window for our query.
        start = start or self.today()
        limit = limit or self.today() + datetime.timedelta(days=90)
        schedule[start] = []
        # Schedule daily tasks.
        for rhythm in self.list_dailies():
            day = start
            while day < limit:
                if not any((True for ev in events.get(rhythm.id, []) if ev.what in ('done', 'defer') and ev.when == day)):
                    if day not in schedule:
                        schedule[day] = []
                    schedule[day].append(rhythm)
                day += ONE_DAY
        # Gather non-daily rhythms and put them in smooth_rhythms.
        rhythms = list(self.list_monthlies()) + list(self.list_week_dailies()) + list(self.list_every_n_days())
        smooth_rhythms = []
        for rhythm in rhythms:
            rhythm_events = sorted(events.get(rhythm.id, []), key=lambda ev: ev.when)
            done_events = [ev.when for ev in rhythm_events if ev.what == 'done']
            defer_events = [ev.when for ev in rhythm_events if ev.what == 'defer']
            if len(done_events) > 0:
                first_beat = continuing_beat(rhythm, start, done_events[-1])
            else:
                first_beat = rhythm.start_beat(start)
            while first_beat in defer_events:
                first_beat = first_beat + ONE_DAY
            smoothing = Smoothing(rhythm, first_beat, start=start, limit=limit)
            schedule_heap_push(smooth_rhythms, smoothing, rhythm, first_beat)
        while len(smooth_rhythms) > 0:
            day, window, periodicity, rhythm_id, smoothing, rhythm = heapq.heappop(smooth_rhythms)
            if all((x < start or x >= limit for x in smoothing.all_options())):
                continue
            slots_per_day = watermarks.get(day, 1)
            spoons_today = spoons.get(day, 5)
            if spoons_today < 0:
                spoons_today = 0
            if spoons_today > 10:
                spoons_today = 10
            spoons_today -= 5
            slots_per_day = math.ceil(slots_per_day * math.pow(2, spoons_today / 5))
            if day not in schedule:
                schedule[day] = []
            slots = schedule[day]
            defer_today = any((ev.what == 'defer' and ev.when == day for ev in events.get(rhythm.id, [])))
            if not defer_today and len(slots) < slots_per_day:
                slots.append(rhythm)
                next_original_beat = continuing_beat(rhythm, start, day)
                smoothing = Smoothing(rhythm, next_original_beat, start=start, limit=limit)
                schedule_heap_push(smooth_rhythms, smoothing, rhythm, next_original_beat)
            elif defer_today or smoothing.remaining_choices:
                if defer_today and (day + ONE_DAY) not in smoothing.remaining_choices:
                    next_day = day + ONE_DAY
                else:
                    next_day = smoothing.shift_one()
                schedule_heap_push(smooth_rhythms, smoothing, rhythm, next_day)
            else:
                options = list(smoothing.all_options())
                low_water_mark = None
                for date in options:
                    if date in watermarks:
                        if low_water_mark is None:
                            low_water_mark = watermarks[date]
                        low_water_mark = min(watermarks[date], low_water_mark)
                    else:
                        low_water_mark = 0
                low_water_mark = low_water_mark or 0
                low_water_mark += 1
                for when in options:
                    watermarks[when] = max(watermarks.get(when, low_water_mark), low_water_mark)
                return self._schedule(start, limit, watermarks)
        return schedule

    def convergence(self):
        today = self.today()
        limit = today + ONE_DAY * 365
        schedule = self.schedule(start=today, limit=limit)
        # Turn the schedule into a dict of { rhythm_id: date }
        first_convergences = collections.defaultdict(lambda: limit)
        for (day, rhythms) in schedule.items():
            for rhythm in rhythms:
                when = min(day, first_convergences[rhythm.id])
                first_convergences[rhythm.id] = when
        # Now figure out the first day at which we'll have completed every rhythm recently.
        converge = today
        for rhythm in self.list_rhythms():
            when = first_convergences.get(rhythm.id, limit)
            if when > converge:
                converge = when
        return converge

    def delinquent(self, start=None):
        start = start or self.today()
        rhythms = list(self.list_rhythms())
        events = collections.defaultdict(list)
        for event in self._conn.execute('SELECT id, dt FROM events WHERE email=? AND what="done"', (self._email,)):
            when = parse_sql_date_or_datetime_as_date(event[1])
            events[event[0]].append(Latest(when=when, what='done'))
        events = dict(events)
        for event_list in events.values():
            event_list.sort()
        # Figure out who is deliqnuent.
        delinquent = set()
        for rhythm in rhythms:
            if rhythm.id in events:
                latest = max(events[rhythm.id])
                if latest.when < start:
                    delinquent.add(rhythm.id)
            else:
                delinquent.add(rhythm.id)
        return [rhythm for rhythm in rhythms if rhythm.id in delinquent]

    def spoons(self, dt, spoons):
        spoons = int(spoons)
        if spoons < 0 or spoons > 10:
            raise ValueError('spoons must not exceed 10')
        self._conn.execute('INSERT OR REPLACE INTO spoons (email, dt, spoons) VALUES (?, ?, ?)', (self._email, dt, spoons))
        self._conn.commit()

    def show_spoons(self):
        spoons = {}
        for dt, num in self._conn.execute('SELECT dt, spoons FROM spoons WHERE email=? ORDER BY dt', (self._email,)):
            dt = parse_sql_date_or_datetime_as_date(dt)
            spoons[dt] = num
        return spoons


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='cadence is a rhythm manager')
    parser.add_argument('--today', default=None, help='the day to consider today')
    subparsers = parser.add_subparsers(dest='cmd', help='command to run')

    # globals
    parser.add_argument('--email', default='robert@example.org', help='email address of the user running cadence')

    # add-daily
    parser_add_daily = subparsers.add_parser('add-daily', help='add a new daily rhythm')
    parser_add_daily.add_argument('desc', nargs='+', help='description of the daily rhythm')

    # add-monthly
    parser_add_monthly = subparsers.add_parser('add-monthly', help='add a new monthly rhythm')
    parser_add_monthly.add_argument('--dotm', default=1, type=int, help='day of the month on which the task occurs')
    parser_add_monthly.add_argument('--slider-before', default=0, type=int, help='days to slide before target date')
    parser_add_monthly.add_argument('--slider-after', default=0, type=int, help='days to slide after target date')
    parser_add_monthly.add_argument('desc', nargs='+', help='description of the monthly rhythm')

    # add-week_daily
    parser_add_week_daily = subparsers.add_parser('add-week-daily', help='add a new week-daily rhythm')
    parser_add_week_daily.add_argument('--dotw', default=0, type=int, help='day of the week on which the task occurs')
    parser_add_week_daily.add_argument('--slider-before', default=0, type=int, help='days to slide before target date')
    parser_add_week_daily.add_argument('--slider-after', default=0, type=int, help='days to slide after target date')
    parser_add_week_daily.add_argument('desc', nargs='+', help='description of the week-daily rhythm')

    # add-every-n-days
    parser_add_every_n_days = subparsers.add_parser('add-every-n-days', help='add a new every-n-days rhythm')
    parser_add_every_n_days.add_argument('--n', default=0, type=int, help='how many days to go between tasks')
    parser_add_every_n_days.add_argument('--slider-before', default=0, type=int, help='days to slide before target date')
    parser_add_every_n_days.add_argument('--slider-after', default=0, type=int, help='days to slide after target date')
    parser_add_every_n_days.add_argument('desc', nargs='+', help='description of the every-n-days rhythm')

    # delete-rhythm
    parser_delete_rhythm = subparsers.add_parser('delete-rhythm', help='completely delete a rhythm and its events')
    parser_delete_rhythm.add_argument('id', help='the ID of the completed rhythm')

    # list-rhythms
    parser_list_rhythms = subparsers.add_parser('list-rhythms', help='list all rhythms')

    # done
    parser_done = subparsers.add_parser('done', help='mark a rhythm as done for today')
    parser_done.add_argument('id', help='the ID of the completed rhythm')

    # defer
    parser_defer = subparsers.add_parser('defer', help='mark a rhythm as defer for today')
    parser_defer.add_argument('id', help='the ID of the completed rhythm')

    # schedule
    today = datetime.datetime.today().date()
    parser_schedule = subparsers.add_parser('schedule', help='schedule the rhythms')
    parser_schedule.add_argument('--start', default=today, help='the start day for this schedule')
    parser_schedule.add_argument('--limit', default=today + 90 * ONE_DAY, help='the limit day for this schedule')

    # convergence
    parser_convergence = subparsers.add_parser('convergence', help='determine when rhythms will converge')

    # delinquent
    parser_delinquent = subparsers.add_parser('delinquent', help='list tasks that are currently delinquent')

    # spoons
    parser_spoons = subparsers.add_parser('spoons', help='set the number of spoons available on a given day')
    parser_spoons.add_argument('--date', default=today, help='which day to update')
    parser_spoons.add_argument('spoons', default=5, help='how many spoons are available')

    # show-spoons
    parser_show_spoons = subparsers.add_parser('show-spoons', help='show which days have configured spoons')

    args = parser.parse_args()
    conn = sqlite3.connect('cadence.db')
    app = CadenceApp(conn, args.email)

    if args.cmd == 'add-daily':
        app.add_daily(' '.join(args.desc))
    elif args.cmd == 'add-monthly':
        app.add_monthly(' '.join(args.desc), args.dotm, args.slider_before, args.slider_after)
    elif args.cmd == 'add-week-daily':
        app.add_week_daily(' '.join(args.desc), args.dotw, args.slider_before, args.slider_after)
    elif args.cmd == 'add-every-n-days':
        app.add_every_n_days(' '.join(args.desc), args.n, args.slider_before, args.slider_after)
    elif args.cmd == 'delete-rhythm':
        app.delete_rhythm(args.id)
    elif args.cmd == 'list-rhythms':
        for rhythm in app.list_rhythms():
            print(rhythm)
    elif args.cmd == 'done':
        app.done(args.id)
    elif args.cmd == 'defer':
        app.defer(args.id)
    elif args.cmd == 'schedule':
        start = parse_sql_date_or_datetime_as_date(args.start)
        limit = parse_sql_date_or_datetime_as_date(args.limit)
        for day, rhythms in app.schedule(start=start, limit=limit).items():
            for rhythm in rhythms:
                print(day, rhythm)
            print()
    elif args.cmd == 'convergence':
        print(app.convergence())
    elif args.cmd == 'delinquent':
        for rhythm in app.delinquent():
            print(rhythm)
    elif args.cmd == 'spoons':
        app.spoons(args.date, args.spoons)
    elif args.cmd == 'show-spoons':
        for dt, spoons in app.show_spoons().items():
            print(dt, spoons)
    else:
        parser.print_help()
        sys.exit(1)
