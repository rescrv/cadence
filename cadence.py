# Copyright 2023 Robert Escriva
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.


import argparse
import base64
import collections
import datetime
import sqlite3
import unittest
import uuid


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

    def next_beat(self, date):
        return date + ONE_DAY

    def next_skipped_beat(self, date):
        return date + ONE_DAY

    def next_new_beat(self, date):
        return date

    def prev_beat(self, date):
        return date - ONE_DAY

    @property
    def slider(self):
        return Slider(0, 0)

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

    def next_beat(self, date):
        if date.day == self.dotm:
            date = date + ONE_DAY
        while date.day != self.dotm:
            date = date + ONE_DAY
        return date

    def next_skipped_beat(self, date):
        return self.next_beat(date + ONE_DAY)

    def next_new_beat(self, date):
        if date.day == self.dotm:
            return date
        return self.next_beat(date)

    def prev_beat(self, date):
        if date.day == self.dotm:
            date = date - ONE_DAY
        while date.day != self.dotm:
            date = date - ONE_DAY
        return date

    @property
    def slider(self):
        return self._slider

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

    def next_beat(self, date):
        if date.weekday() == self.dotw:
            date = date + ONE_DAY
        while date.weekday() != self.dotw:
            date = date + ONE_DAY
        return date

    def next_skipped_beat(self, date):
        return self.next_beat(date + ONE_DAY)

    def next_new_beat(self, date):
        if date.weekday() == self.dotw:
            return date
        return self.next_beat(date)

    def prev_beat(self, date):
        if date.weekday() == self.dotw:
            date = date - ONE_DAY
        while date.weekday() != self.dotw:
            date = date - ONE_DAY
        return date

    @property
    def slider(self):
        return self._slider

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

    def next_beat(self, date):
        return date + datetime.timedelta(days=self.n)

    def next_skipped_beat(self, date):
        return date + ONE_DAY

    def next_new_beat(self, date):
        return date

    def prev_beat(self, date):
        return date - datetime.timedelta(days=self.n)

    @property
    def slider(self):
        return self._slider

    def __str__(self):
        return '{} n:{} slider:{},{} {}'.format(self.id, self.n, self.slider.before, self.slider.after, self.desc)


def starting_beat(rhythm, start, last_seen):
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


def create_table_if_not_exist(conn, table_name, table_definition):
    cursor = conn.cursor()
    query = "SELECT name FROM sqlite_master WHERE type='table' AND name=?"
    if len(list(cursor.execute(query, (table_name,)))) == 0:
        cursor.execute(table_definition)
    del cursor


def generate_id():
    return base64.urlsafe_b64encode(uuid.uuid4().bytes).rstrip(b'=').decode('ascii')


class CadenceApp:

    def __init__(self, conn, email):
        self._conn = conn
        self._email = email
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
                ts timestamp NOT NULL,
                PRIMARY KEY (email, id, ts)
                )''')

    def add_daily(self, desc):
        id = generate_id()
        self._conn.execute('INSERT INTO rhythms (email, id, desc) VALUES (?, ?, ?)', (self._email, id, desc))
        self._conn.execute('INSERT INTO dailies (email, id) VALUES (?, ?)', (self._email, id))
        self._conn.commit()
        return id

    def add_monthly(self, desc, dotm, slider_before=0, slider_after=0):
        if dotm < 1 or dotm > 31:
            raise ValueError('day of the month must be [1, 31]')
        id = generate_id()
        self._conn.execute('INSERT INTO rhythms (email, id, desc) VALUES (?, ?, ?)', (self._email, id, desc))
        self._conn.execute('INSERT INTO monthlies (email, id, dotm, slider_before, slider_after) VALUES (?, ?, ?, ?, ?)',
                           (self._email, id, dotm, slider_before, slider_after))
        self._conn.commit()
        return id

    def add_week_daily(self, desc, dotw, slider_before=0, slider_after=0):
        if dotw < 0 or dotw > 6:
            raise ValueError('day of the week must be [0, 6]')
        id = generate_id()
        self._conn.execute('INSERT INTO rhythms (email, id, desc) VALUES (?, ?, ?)', (self._email, id, desc))
        self._conn.execute('INSERT INTO week_dailies (email, id, dotw, slider_before, slider_after) VALUES (?, ?, ?, ?, ?)',
                           (self._email, id, dotw, slider_before, slider_after))
        self._conn.commit()
        return id

    def add_every_n_days(self, desc, n, slider_before=0, slider_after=0):
        if n < 2:
            raise ValueError('n cannot be smaller than 2')
        id = generate_id()
        self._conn.execute('INSERT INTO rhythms (email, id, desc) VALUES (?, ?, ?)', (self._email, id, desc))
        self._conn.execute('INSERT INTO every_n_days (email, id, n, slider_before, slider_after) VALUES (?, ?, ?, ?, ?)',
                           (self._email, id, n, slider_before, slider_after))
        self._conn.commit()
        return id

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
        # TODO(rescrv):  Put these in local time zone for user.
        when = datetime.datetime.now()
        self._conn.execute('INSERT INTO events (email, id, what, ts) VALUES (?, ?, "done", ?)', (self._email, id, when))
        self._conn.commit()

    def skip(self, id, when=None):
        # TODO(rescrv):  Put these in local time zone for user.
        when = when or datetime.datetime.now()
        self._conn.execute('INSERT INTO events (email, id, what, ts) VALUES (?, ?, "skip", ?)', (self._email, id, when))
        self._conn.commit()

    def schedule(self, start=None, limit=None):
        rhythms = list(self.list_rhythms())
        events = collections.defaultdict(list)
        # TODO(rescrv):  Once further out, truncate this to latest events of each type.
        for event in self._conn.execute('SELECT id, what, ts FROM events WHERE email=?', (self._email,)):
            events[event[0]].append(Latest(when=event[2], what=event[1]))
        events = dict(events)
        for event_list in events.values():
            event_list.sort()
        start = start or datetime.date.today()
        limit = limit or datetime.date.today() + datetime.timedelta(days=30)
        latest_by_rhythm = {}
        for rhythm in rhythms:
            if rhythm.id in events:
                try:
                    when = datetime.datetime.strptime(events[rhythm.id][-1].when, '%Y-%m-%d %H:%M:%S.%f')
                except ValueError:
                    try:
                        when = datetime.datetime.strptime(events[rhythm.id][-1].when, '%Y-%m-%d %H:%M:%S')
                    except ValueError:
                        when = datetime.datetime.strptime(events[rhythm.id][-1].when, '%Y-%m-%d')
                latest = Latest(when=when.date(), what=events[rhythm.id][-1].what)
            else:
                # If it hasn't been done before, act as if it's a skip.
                latest = Latest(when=rhythm.prev_beat(start), what='new')
            latest_by_rhythm[rhythm] = latest
        schedule = {}
        day = start
        while day < limit:
            schedule[day] = []
            for rhythm in rhythms:
                latest = latest_by_rhythm[rhythm]
                if latest.what == 'new':
                    next_beat = rhythm.next_new_beat(start)
                elif latest.what == 'skip':
                    next_beat = rhythm.next_skipped_beat(latest.when)
                else:
                    next_beat = rhythm.next_beat(latest.when)
                if next_beat <= day:
                    schedule[day].append(rhythm)
                    latest_by_rhythm[rhythm] = Latest(when=day, what='done')
            day += ONE_DAY
        return schedule


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='cadence is a rhythm manager')
    subparsers = parser.add_subparsers(dest='cmd', help='command to run')

    # globals
    parser.add_argument('--email', default='robert@rescrv.net', help='email address of the user running cadence')

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

    # list-rhythms
    parser_list_rhythms = subparsers.add_parser('list-rhythms', help='list all rhythms')

    # done
    parser_done = subparsers.add_parser('done', help='mark a rhythm as done for now')
    parser_done.add_argument('id', help='the ID of the completed rhythm')

    # skip
    parser_skip = subparsers.add_parser('skip', help='mark a rhythm as skip for now')
    parser_skip.add_argument('id', help='the ID of the completed rhythm')

    # schedule
    parser_schedule = subparsers.add_parser('schedule', help='schedule the rhythms')

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
    elif args.cmd == 'list-rhythms':
        for rhythm in app.list_rhythms():
            print(rhythm)
    elif args.cmd == 'done':
        app.done(args.id)
    elif args.cmd == 'skip':
        app.skip(args.id)
    elif args.cmd == 'schedule':
        for day, rhythms in app.schedule().items():
            for rhythm in rhythms:
                print(day, rhythm)
            print()
    else:
        raise RuntimeError('unknown command {}'.format(args.cmd))
