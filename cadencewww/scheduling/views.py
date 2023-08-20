import datetime
import re
import sqlite3
import zoneinfo

from django.contrib.auth.decorators import login_required
from django.contrib.auth.models import User
from django.shortcuts import render, redirect
from django.views.decorators.http import require_POST

import cadence
from cadence import CadenceApp

@login_required
def index(request):
    schedule = request.cadence_app.schedule()
    context = { 'email': request.user.email, 'schedule': schedule }
    return render(request, 'scheduling/index.html', context)

@login_required
def delinquent(request):
    delinquent = request.cadence_app.delinquent()
    context = { 'email': request.user.email, 'delinquent': delinquent }
    return render(request, 'scheduling/delinquent.html', context)

def mark(request, what):
    rhythm_id = request.POST.get('rhythm_id', '')
    when = request.POST.get('rhythm_when', '')
    if rhythm_id:
        what(request.cadence_app, rhythm_id, when)
        return redirect('full-schedule')
    else:
        assert False

@login_required
@require_POST
def done(request):
    return mark(request, lambda cadence, rhythm_id, when: cadence.done(rhythm_id, when))

@login_required
@require_POST
def defer(request):
    return mark(request, lambda cadence, rhythm_id, when: cadence.defer(rhythm_id, when))

@login_required
def spoons(request):
    days = dict([(datetime.datetime.today().date() + i * cadence.ONE_DAY, 5) for i in range(90)])
    if request.method == "POST":
        for k, v in request.POST.items():
            date = re.match('date_(\d{4})-(\d{2})-(\d{2})', k)
            if date:
                spoons = int(v)
                date = datetime.date(int(date.group(1), 10), int(date.group(2), 10), int(date.group(3), 10))
                request.cadence_app.spoons(date, spoons)
    for dt, spoons in request.cadence_app.show_spoons().items():
        days[dt] = spoons
    context = { 'email': request.user.email, 'days': days }
    return render(request, 'scheduling/spoons.html', context)
