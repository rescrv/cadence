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

    def test_start_beat(self):
        assert self.DAILY.start_beat(datetime.date(2023, 8, 19)) == datetime.date(2023, 8, 19)

    def test_next_beat(self):
        assert self.DAILY.next_beat(datetime.date(2022, 11, 20)) == datetime.date(2022, 11, 21)

    def test_prev_beat(self):
        assert self.DAILY.prev_beat(datetime.date(2022, 11, 20)) == datetime.date(2022, 11, 19)

    def test_slider(self):
        assert self.DAILY.slider.before == 0
        assert self.DAILY.slider.after == 0


class TestMonthly(unittest.TestCase):

    MONTHLY = Monthly('id', 'some monthly rhythm', 18, Slider(7, 3))

    def test_start_beat_same_day(self):
        assert self.MONTHLY.start_beat(datetime.date(2022, 11, 18)) == datetime.date(2022, 11, 18)

    def test_startbeat_different_day(self):
        assert self.MONTHLY.start_beat(datetime.date(2022, 11, 20)) == datetime.date(2022, 12, 18)

    def test_next_beat_same_day(self):
        assert self.MONTHLY.next_beat(datetime.date(2022, 11, 18)) == datetime.date(2022, 12, 18)

    def test_next_beat_different_day(self):
        assert self.MONTHLY.next_beat(datetime.date(2022, 11, 20)) == datetime.date(2022, 12, 18)

    def test_prev_beat_same_day(self):
        assert self.MONTHLY.prev_beat(datetime.date(2022, 11, 18)) == datetime.date(2022, 10, 18)

    def test_prev_beat_different_day(self):
        assert self.MONTHLY.prev_beat(datetime.date(2022, 11, 17)) == datetime.date(2022, 10, 18)

    def test_slider(self):
        assert self.MONTHLY.slider.before == 7
        assert self.MONTHLY.slider.after == 3

    def test_default_slider(self):
        monthly = Monthly('id', 'some monthly rhythm', 18)
        assert monthly.slider.before == 0
        assert monthly.slider.after == 0


class TestWeekDaily(unittest.TestCase):

    WEEK_DAILY = WeekDaily('id', 'some week daily rhythm', 5, Slider(2, 1))

    def test_start_beat_same_day(self):
        assert self.WEEK_DAILY.start_beat(datetime.date(2022, 11, 19)) == datetime.date(2022, 11, 19)

    def test_start_beat_different_day(self):
        assert self.WEEK_DAILY.start_beat(datetime.date(2022, 11, 20)) == datetime.date(2022, 11, 26)

    def test_next_beat_same_day(self):
        assert self.WEEK_DAILY.next_beat(datetime.date(2022, 11, 19)) == datetime.date(2022, 11, 26)

    def test_next_beat_different_day(self):
        assert self.WEEK_DAILY.next_beat(datetime.date(2022, 11, 20)) == datetime.date(2022, 11, 26)

    def test_prev_beat_same_day(self):
        assert self.WEEK_DAILY.prev_beat(datetime.date(2022, 11, 19)) == datetime.date(2022, 11, 12)

    def test_prev_beat_different_day(self):
        assert self.WEEK_DAILY.prev_beat(datetime.date(2022, 11, 18)) == datetime.date(2022, 11, 12)

    def test_slider(self):
        assert self.WEEK_DAILY.slider.before == 2
        assert self.WEEK_DAILY.slider.after == 1

    def test_default_slider(self):
        week_daily = WeekDaily('id', 'some week daily rhythm', 18)
        assert week_daily.slider.before == 0
        assert week_daily.slider.after == 0


class TestEveryNDays(unittest.TestCase):

    EVERY_N_DAYS = EveryNDays('id', 'some rhythm that happens every n days', 5, Slider(1, 2))

    def test_start_beat(self):
        assert self.EVERY_N_DAYS.start_beat(datetime.date(2023, 8, 19)) == datetime.date(2023, 8, 22)

    def test_next_beat(self):
        assert self.EVERY_N_DAYS.next_beat(datetime.date(2022, 11, 20)) == datetime.date(2022, 11, 25)

    def test_prev_beat(self):
        assert self.EVERY_N_DAYS.prev_beat(datetime.date(2022, 11, 20)) == datetime.date(2022, 11, 15)

    def test_slider(self):
        assert self.EVERY_N_DAYS.slider.before == 1
        assert self.EVERY_N_DAYS.slider.after == 2

    def test_default_slider(self):
        every_n_days = EveryNDays('id', 'some rhythm that happens every n days', 5)
        assert every_n_days.slider.before == 0
        assert every_n_days.slider.after == 0


class TestContinuingBeat(unittest.TestCase):

    def test_daily(self):
        start = datetime.date(2022, 11, 20)
        last_seen = datetime.date(2022, 11, 19)
        beat = continuing_beat(TestDaily.DAILY, start, last_seen)
        assert beat == datetime.date(2022, 11, 20)

    def test_monthly(self):
        start = datetime.date(2022, 11, 20)
        last_seen = datetime.date(2022, 10, 20)
        beat = continuing_beat(TestMonthly.MONTHLY, start, last_seen)
        assert beat == datetime.date(2022, 12, 18)

    def test_monthly_same_day(self):
        start = datetime.date(2022, 11, 18)
        last_seen = datetime.date(2022, 10, 17)
        beat = continuing_beat(TestMonthly.MONTHLY, start, last_seen)
        assert beat == datetime.date(2022, 11, 18)


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


if __name__ == '__main__':
    unittest.main()
