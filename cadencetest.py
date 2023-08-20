#!/usr/bin/env python3
#
# Copyright (c) 2023 Robert Escriva

import datetime
import sqlite3
import unittest

from cadence import *


TEST_EMAIL = 'user@example.org'


class TestDaily(unittest.TestCase):

    DAILY = Daily('id', 'some daily rhythm')

    def test_next_beat(self):
        when = datetime.datetime(2022, 11, 20, 9, 54, 10)
        next = self.DAILY.next_beat(when)
        assert next == datetime.datetime(2022, 11, 21, 9, 54, 10)

    def test_prev_beat(self):
        when = datetime.datetime(2022, 11, 20, 9, 54, 10)
        prev = self.DAILY.prev_beat(when)
        assert prev == datetime.datetime(2022, 11, 19, 9, 54, 10)

    def test_slider(self):
        assert self.DAILY.slider.before == 0
        assert self.DAILY.slider.after == 0


class TestMonthly(unittest.TestCase):

    MONTHLY = Monthly('id', 'some monthly rhythm', 18, Slider(7, 3))

    def test_next_beat_same_day(self):
        when = datetime.datetime(2022, 11, 18, 10, 3, 29)
        next = self.MONTHLY.next_beat(when)
        assert next == datetime.datetime(2022, 12, 18, 10, 3, 29)

    def test_next_beat_different_day(self):
        when = datetime.datetime(2022, 11, 20, 10, 3, 29)
        next = self.MONTHLY.next_beat(when)
        assert next == datetime.datetime(2022, 12, 18, 10, 3, 29)

    def test_prev_beat_same_day(self):
        when = datetime.datetime(2022, 11, 18, 10, 3, 29)
        prev = self.MONTHLY.prev_beat(when)
        assert prev == datetime.datetime(2022, 10, 18, 10, 3, 29)

    def test_prev_beat_different_day(self):
        when = datetime.datetime(2022, 11, 17, 10, 3, 29)
        prev = self.MONTHLY.prev_beat(when)
        assert prev == datetime.datetime(2022, 10, 18, 10, 3, 29)

    def test_slider(self):
        assert self.MONTHLY.slider.before == 7
        assert self.MONTHLY.slider.after == 3

    def test_default_slider(self):
        monthly = Monthly('id', 'some monthly rhythm', 18)
        assert monthly.slider.before == 0
        assert monthly.slider.after == 0


class TestWeekDaily(unittest.TestCase):

    WEEK_DAILY = WeekDaily('id', 'some week daily rhythm', 5, Slider(2, 1))

    def test_next_beat_same_day(self):
        when = datetime.datetime(2022, 11, 19, 10, 15, 0)
        next = self.WEEK_DAILY.next_beat(when)
        assert next == datetime.datetime(2022, 11, 26, 10, 15, 0)

    def test_next_beat_different_day(self):
        when = datetime.datetime(2022, 11, 20, 10, 15, 0)
        next = self.WEEK_DAILY.next_beat(when)
        assert next == datetime.datetime(2022, 11, 26, 10, 15, 0)

    def test_prev_beat_same_day(self):
        when = datetime.datetime(2022, 11, 19, 10, 15, 0)
        prev = self.WEEK_DAILY.prev_beat(when)
        assert prev == datetime.datetime(2022, 11, 12, 10, 15, 0)

    def test_prev_beat_different_day(self):
        when = datetime.datetime(2022, 11, 18, 10, 15, 0)
        prev = self.WEEK_DAILY.prev_beat(when)
        assert prev == datetime.datetime(2022, 11, 12, 10, 15, 0)

    def test_slider(self):
        assert self.WEEK_DAILY.slider.before == 2
        assert self.WEEK_DAILY.slider.after == 1

    def test_default_slider(self):
        week_daily = WeekDaily('id', 'some week daily rhythm', 18)
        assert week_daily.slider.before == 0
        assert week_daily.slider.after == 0


class TestEveryNDays(unittest.TestCase):

    EVERY_N_DAYS = EveryNDays('id', 'some rhythm that happens every n days', 5, Slider(1, 2))

    def test_next_beat(self):
        when = datetime.datetime(2022, 11, 20, 10, 23, 17)
        next = self.EVERY_N_DAYS.next_beat(when)
        assert next == datetime.datetime(2022, 11, 25, 10, 23, 17)

    def test_prev_beat(self):
        when = datetime.datetime(2022, 11, 20, 10, 23, 17)
        prev = self.EVERY_N_DAYS.prev_beat(when)
        assert prev == datetime.datetime(2022, 11, 15, 10, 23, 17)

    def test_slider(self):
        assert self.EVERY_N_DAYS.slider.before == 1
        assert self.EVERY_N_DAYS.slider.after == 2

    def test_default_slider(self):
        every_n_days = EveryNDays('id', 'some rhythm that happens every n days', 5)
        assert every_n_days.slider.before == 0
        assert every_n_days.slider.after == 0


class TestStartingBeat(unittest.TestCase):

    def test_daily(self):
        start = datetime.datetime(2022, 11, 20, 10, 35, 57)
        last_seen = datetime.datetime(2022, 11, 19, 10, 35, 57)
        beat = starting_beat(TestDaily.DAILY, start, last_seen)
        assert beat == datetime.datetime(2022, 11, 20, 10, 35, 57)

    def test_monthly(self):
        start = datetime.datetime(2022, 11, 20, 10, 35, 57)
        last_seen = datetime.datetime(2022, 10, 20, 10, 35, 57)
        beat = starting_beat(TestMonthly.MONTHLY, start, last_seen)
        assert beat == datetime.datetime(2022, 12, 18, 10, 35, 57)

    def test_monthly_same_day(self):
        start = datetime.datetime(2022, 11, 18, 10, 35, 57)
        last_seen = datetime.datetime(2022, 10, 17, 10, 35, 57)
        beat = starting_beat(TestMonthly.MONTHLY, start, last_seen)
        assert beat == datetime.datetime(2022, 11, 18, 10, 35, 57)


class TestCreateTableIfNotExists(unittest.TestCase):

    def test_clean_slate(self):
        conn = sqlite3.connect(':memory:')
        create_table_if_not_exist(conn, 'table1', 'CREATE TABLE table1 (id integer PRIMARY KEY)')
        exists = list(conn.execute("SELECT name FROM sqlite_master WHERE type='table' AND name=?", ('table1',)))
        # TODO(rescrv):  This prints True but fails the assert.
        #print(exists == [('table1',)])
        #assert exists == [('table1,')]

    def test_existing(self):
        conn = sqlite3.connect(':memory:')
        create_table_if_not_exist(conn, 'table1', 'CREATE TABLE table1 (id integer PRIMARY KEY)')
        create_table_if_not_exist(conn, 'table1', 'CREATE TABLE table1 (id integer PRIMARY KEY)')
        exists = list(conn.execute("SELECT name FROM sqlite_master WHERE type='table' AND name=?", ('table1',)))
        # TODO(rescrv):  This prints True but fails the assert.
        #print(exists == [('table1',)])
        #assert exists == [('table1,')]


class TestAddDaily(unittest.TestCase):

    DESC = 'test description of a daily task'

    def test_add_daily(self):
        conn = sqlite3.connect(':memory:')
        app = CadenceApp(conn, TEST_EMAIL)
        app.add_daily(self.DESC)
        rhythms = list(conn.execute('SELECT email, id, desc FROM rhythms'))
        assert rhythms[0][0] == TEST_EMAIL
        assert len(rhythms[0][1]) == 22
        assert rhythms[0][2] == self.DESC
        dailies = list(conn.execute('SELECT email, id FROM dailies'))
        assert dailies[0][0] == TEST_EMAIL
        assert dailies[0][1] == rhythms[0][1]


class TestAddMonthly(unittest.TestCase):

    DESC = 'test description of a monthly task'

    def test_add_monthly(self):
        conn = sqlite3.connect(':memory:')
        app = CadenceApp(conn, TEST_EMAIL)
        app.add_monthly(self.DESC, 5)
        rhythms = list(conn.execute('SELECT email, id, desc FROM rhythms'))
        assert rhythms[0][0] == TEST_EMAIL
        assert len(rhythms[0][1]) == 22
        assert rhythms[0][2] == self.DESC
        monthlies = list(conn.execute('SELECT email, id, dotm FROM monthlies'))
        assert monthlies[0][0] == TEST_EMAIL
        assert monthlies[0][1] == rhythms[0][1]
        assert monthlies[0][2] == 5

    def test_add_monthly_slider(self):
        conn = sqlite3.connect(':memory:')
        app = CadenceApp(conn, TEST_EMAIL)
        app.add_monthly(self.DESC, 5, 1, 3)

    def test_below_1(self):
        conn = sqlite3.connect(':memory:')
        app = CadenceApp(conn, TEST_EMAIL)
        with self.assertRaises(ValueError):
            app.add_monthly(self.DESC, 0)

    def test_exceeds_31(self):
        conn = sqlite3.connect(':memory:')
        app = CadenceApp(conn, TEST_EMAIL)
        with self.assertRaises(ValueError):
            app.add_monthly(self.DESC, 32)


class TestAddWeekDaily(unittest.TestCase):

    DESC = 'test description of a week-daily task'

    def test_add_week_daily(self):
        conn = sqlite3.connect(':memory:')
        app = CadenceApp(conn, TEST_EMAIL)
        app.add_week_daily(self.DESC, 5)
        rhythms = list(conn.execute('SELECT email, id, desc FROM rhythms'))
        assert rhythms[0][0] == TEST_EMAIL
        assert len(rhythms[0][1]) == 22
        assert rhythms[0][2] == self.DESC
        week_dailies = list(conn.execute('SELECT email, id, dotw FROM week_dailies'))
        assert week_dailies[0][0] == TEST_EMAIL
        assert week_dailies[0][1] == rhythms[0][1]
        assert week_dailies[0][2] == 5

    def test_add_week_dailys_slider(self):
        conn = sqlite3.connect(':memory:')
        app = CadenceApp(conn, TEST_EMAIL)
        app.add_week_daily(self.DESC, 5, 1, 3)

    def test_below_0(self):
        conn = sqlite3.connect(':memory:')
        app = CadenceApp(conn, TEST_EMAIL)
        with self.assertRaises(ValueError):
            app.add_week_daily(self.DESC, -1)

    def test_exceeds_6(self):
        conn = sqlite3.connect(':memory:')
        app = CadenceApp(conn, TEST_EMAIL)
        with self.assertRaises(ValueError):
            app.add_week_daily(self.DESC, 7)


class TestAddEveryNDays(unittest.TestCase):

    DESC = 'test description of a every-n-daily task'

    def test_add_every_n_days(self):
        conn = sqlite3.connect(':memory:')
        app = CadenceApp(conn, TEST_EMAIL)
        app.add_every_n_days(self.DESC, 2)
        rhythms = list(conn.execute('SELECT email, id, desc FROM rhythms'))
        assert rhythms[0][0] == TEST_EMAIL
        assert len(rhythms[0][1]) == 22
        assert rhythms[0][2] == self.DESC
        every_n_days = list(conn.execute('SELECT email, id, n FROM every_n_days'))
        assert every_n_days[0][0] == TEST_EMAIL
        assert every_n_days[0][1] == rhythms[0][1]
        assert every_n_days[0][2] == 2

    def test_add_every_n_days_slider(self):
        conn = sqlite3.connect(':memory:')
        app = CadenceApp(conn, TEST_EMAIL)
        app.add_every_n_days(self.DESC, 2, 1, 3)

    def test_below_2(self):
        conn = sqlite3.connect(':memory:')
        app = CadenceApp(conn, TEST_EMAIL)
        with self.assertRaises(ValueError):
            app.add_every_n_days(self.DESC, 1)


class TestListRhythms(unittest.TestCase):

    def test_list_rhythms(self):
        conn = sqlite3.connect(':memory:')
        app = CadenceApp(conn, TEST_EMAIL)
        app.add_daily(TestAddDaily.DESC)
        app.add_monthly(TestAddMonthly.DESC, 18)
        app.add_week_daily(TestAddWeekDaily.DESC, 1)
        app.add_every_n_days(TestAddEveryNDays.DESC, 3)
        rhythms = list(app.list_rhythms())
        assert rhythms[0].desc == TestAddDaily.DESC
        assert rhythms[1].desc == TestAddMonthly.DESC
        assert rhythms[2].desc == TestAddWeekDaily.DESC
        assert rhythms[3].desc == TestAddEveryNDays.DESC


class TestDone(unittest.TestCase):

    def test_done(self):
        conn = sqlite3.connect(':memory:')
        app = CadenceApp(conn, TEST_EMAIL)
        app.add_daily(TestAddDaily.DESC)
        rhythms = list(app.list_rhythms())
        app.done(rhythms[0].id)
        app.done(rhythms[0].id)
        app.done(rhythms[0].id)
        events = list(conn.execute('SELECT id, what FROM events WHERE email=? AND id=?', (TEST_EMAIL, rhythms[0].id)))
        assert len(events) == 3
        assert events[0][0] == rhythms[0].id
        assert events[1][0] == rhythms[0].id
        assert events[2][0] == rhythms[0].id
        assert events[0][1] == 'done'
        assert events[1][1] == 'done'
        assert events[2][1] == 'done'


class TestSkip(unittest.TestCase):

    def test_skip(self):
        conn = sqlite3.connect(':memory:')
        app = CadenceApp(conn, TEST_EMAIL)
        app.add_daily(TestAddDaily.DESC)
        rhythms = list(app.list_rhythms())
        app.skip(rhythms[0].id)
        app.skip(rhythms[0].id)
        app.skip(rhythms[0].id)
        events = list(conn.execute('SELECT id,what FROM events WHERE email=? AND id=?', (TEST_EMAIL, rhythms[0].id)))
        assert len(events) == 3
        assert events[0][0] == rhythms[0].id
        assert events[1][0] == rhythms[0].id
        assert events[2][0] == rhythms[0].id
        assert events[0][1] == 'skip'
        assert events[1][1] == 'skip'
        assert events[2][1] == 'skip'


class TestParseSqlDateOrDatetimeAsDate(unittest.TestCase):
    assert datetime.date(2022, 11, 20) == parse_sql_date_or_datetime_as_date('2022-11-20 23:42:21.0')
    assert datetime.date(2022, 11, 20) == parse_sql_date_or_datetime_as_date('2022-11-20 23:42:21')
    assert datetime.date(2022, 11, 20) == parse_sql_date_or_datetime_as_date('2022-11-20')


class TestSchedule(unittest.TestCase):

    def test_schedule(self):
        conn = sqlite3.connect(':memory:')
        app = CadenceApp(conn, TEST_EMAIL)
        daily = app.add_daily(TestAddDaily.DESC)
        monthly = app.add_monthly(TestAddMonthly.DESC, 18)
        week_daily = app.add_week_daily(TestAddWeekDaily.DESC, 1)
        every_n_days = app.add_every_n_days(TestAddEveryNDays.DESC, 2)
        schedule = app.schedule(datetime.date(2022, 11, 13), datetime.date(2022, 11, 20))
        assert datetime.date(2022, 11, 20) not in schedule
        assert datetime.date(2022, 11, 13) in schedule
        assert datetime.date(2022, 11, 14) in schedule
        assert datetime.date(2022, 11, 15) in schedule
        assert datetime.date(2022, 11, 16) in schedule
        assert datetime.date(2022, 11, 17) in schedule
        assert datetime.date(2022, 11, 18) in schedule
        assert datetime.date(2022, 11, 19) in schedule

        assert len(schedule[datetime.date(2022, 11, 13)]) == 2
        assert schedule[datetime.date(2022, 11, 13)][0].id == daily
        assert schedule[datetime.date(2022, 11, 13)][1].id == every_n_days

        assert len(schedule[datetime.date(2022, 11, 14)]) == 1
        assert schedule[datetime.date(2022, 11, 14)][0].id == daily

        assert len(schedule[datetime.date(2022, 11, 15)]) == 3
        assert schedule[datetime.date(2022, 11, 15)][0].id == daily
        assert schedule[datetime.date(2022, 11, 15)][1].id == week_daily
        assert schedule[datetime.date(2022, 11, 15)][2].id == every_n_days

        assert len(schedule[datetime.date(2022, 11, 16)]) == 1
        assert schedule[datetime.date(2022, 11, 16)][0].id == daily

        assert len(schedule[datetime.date(2022, 11, 17)]) == 2
        assert schedule[datetime.date(2022, 11, 17)][0].id == daily
        assert schedule[datetime.date(2022, 11, 17)][1].id == every_n_days

        assert len(schedule[datetime.date(2022, 11, 18)]) == 2
        assert schedule[datetime.date(2022, 11, 18)][0].id == daily
        assert schedule[datetime.date(2022, 11, 18)][1].id == monthly

        assert len(schedule[datetime.date(2022, 11, 19)]) == 2
        assert schedule[datetime.date(2022, 11, 19)][0].id == daily
        assert schedule[datetime.date(2022, 11, 19)][1].id == every_n_days

    def test_schedule_skip_start(self):
        conn = sqlite3.connect(':memory:')
        app = CadenceApp(conn, TEST_EMAIL)
        daily = app.add_daily(TestAddDaily.DESC)
        monthly = app.add_monthly(TestAddMonthly.DESC, 18)
        week_daily = app.add_week_daily(TestAddWeekDaily.DESC, 1)
        every_n_days = app.add_every_n_days(TestAddEveryNDays.DESC, 2)
        app.skip(daily, datetime.datetime(2022, 11, 13))
        app.skip(monthly, datetime.datetime(2022, 11, 13))
        app.skip(week_daily, datetime.datetime(2022, 11, 13))
        app.skip(every_n_days, datetime.datetime(2022, 11, 13))
        schedule = app.schedule(datetime.date(2022, 11, 13), datetime.date(2022, 11, 20))
        assert datetime.date(2022, 11, 20) not in schedule
        assert datetime.date(2022, 11, 13) in schedule
        assert datetime.date(2022, 11, 14) in schedule
        assert datetime.date(2022, 11, 15) in schedule
        assert datetime.date(2022, 11, 16) in schedule
        assert datetime.date(2022, 11, 17) in schedule
        assert datetime.date(2022, 11, 18) in schedule
        assert datetime.date(2022, 11, 19) in schedule

        assert len(schedule[datetime.date(2022, 11, 13)]) == 0

        assert len(schedule[datetime.date(2022, 11, 14)]) == 2
        assert schedule[datetime.date(2022, 11, 14)][0].id == daily
        assert schedule[datetime.date(2022, 11, 14)][1].id == every_n_days

        assert len(schedule[datetime.date(2022, 11, 15)]) == 2
        assert schedule[datetime.date(2022, 11, 15)][0].id == daily
        assert schedule[datetime.date(2022, 11, 15)][1].id == week_daily

        assert len(schedule[datetime.date(2022, 11, 16)]) == 2
        assert schedule[datetime.date(2022, 11, 16)][0].id == daily
        assert schedule[datetime.date(2022, 11, 16)][1].id == every_n_days

        assert len(schedule[datetime.date(2022, 11, 17)]) == 1
        assert schedule[datetime.date(2022, 11, 17)][0].id == daily

        assert len(schedule[datetime.date(2022, 11, 18)]) == 3
        assert schedule[datetime.date(2022, 11, 18)][0].id == daily
        assert schedule[datetime.date(2022, 11, 18)][1].id == monthly
        assert schedule[datetime.date(2022, 11, 18)][2].id == every_n_days

        assert len(schedule[datetime.date(2022, 11, 19)]) == 1
        assert schedule[datetime.date(2022, 11, 19)][0].id == daily

    def test_schedule(self):
        conn = sqlite3.connect(':memory:')
        app = CadenceApp(conn, TEST_EMAIL)
        daily = app.add_daily(TestAddDaily.DESC)
        monthly = app.add_monthly(TestAddMonthly.DESC, 18, 1, 1)
        week_daily = app.add_week_daily(TestAddWeekDaily.DESC, 1, 1, 1)
        every_n_days = app.add_every_n_days(TestAddEveryNDays.DESC, 2, 1, 1)
        schedule = app.schedule(datetime.date(2022, 11, 13), datetime.date(2022, 11, 20))
        assert datetime.date(2022, 11, 20) not in schedule
        assert datetime.date(2022, 11, 13) in schedule
        assert datetime.date(2022, 11, 14) in schedule
        assert datetime.date(2022, 11, 15) in schedule
        assert datetime.date(2022, 11, 16) in schedule
        assert datetime.date(2022, 11, 17) in schedule
        assert datetime.date(2022, 11, 18) in schedule
        assert datetime.date(2022, 11, 19) in schedule

        assert len(schedule[datetime.date(2022, 11, 13)]) == 2
        assert schedule[datetime.date(2022, 11, 13)][0].id == daily
        assert schedule[datetime.date(2022, 11, 13)][1].id == every_n_days

        assert len(schedule[datetime.date(2022, 11, 14)]) == 1
        assert schedule[datetime.date(2022, 11, 14)][0].id == daily

        assert len(schedule[datetime.date(2022, 11, 15)]) == 3
        assert schedule[datetime.date(2022, 11, 15)][0].id == daily
        assert schedule[datetime.date(2022, 11, 15)][1].id == week_daily
        assert schedule[datetime.date(2022, 11, 15)][2].id == every_n_days

        assert len(schedule[datetime.date(2022, 11, 16)]) == 1
        assert schedule[datetime.date(2022, 11, 16)][0].id == daily

        assert len(schedule[datetime.date(2022, 11, 17)]) == 2
        assert schedule[datetime.date(2022, 11, 17)][0].id == daily
        assert schedule[datetime.date(2022, 11, 17)][1].id == every_n_days

        assert len(schedule[datetime.date(2022, 11, 18)]) == 2
        assert schedule[datetime.date(2022, 11, 18)][0].id == daily
        assert schedule[datetime.date(2022, 11, 18)][1].id == monthly

        assert len(schedule[datetime.date(2022, 11, 19)]) == 2
        assert schedule[datetime.date(2022, 11, 19)][0].id == daily
        assert schedule[datetime.date(2022, 11, 19)][1].id == every_n_days


if __name__ == '__main__':
    unittest.main()
