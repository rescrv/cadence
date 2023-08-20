import datetime
import sqlite3

from django.contrib.auth.decorators import login_required
from django.contrib.auth.models import User
from django.http import Http404
from django.shortcuts import render, redirect
from django.views.decorators.http import require_POST

from cadence import CadenceApp
import cadence

from . import forms

@login_required
def index(request):
    rhythms = list(request.cadence_app.list_rhythms())
    context = { 'email': request.user.email, 'rhythms': rhythms }
    return render(request, 'rhythms/index.html', context)

def form_for_rhythm(rhythm):
    if isinstance(rhythm, cadence.Daily):
        return forms.DailyRhythmForm
    elif isinstance(rhythm, cadence.Monthly):
        return forms.MonthlyRhythmForm
    elif isinstance(rhythm, cadence.WeekDaily):
        return forms.WeekDailyRhythmForm
    elif isinstance(rhythm, cadence.EveryNDays):
        return forms.EveryNDaysRhythmForm
    else:
        # TODO(rescrv): Alert on this particular 404.
        raise Http404

@login_required
def add(request):
    context = { 'email': request.user.email }
    return render(request, 'rhythms/add.html', context)

def _add_rhythm(request, Form, templ, save):
    if request.method == "POST":
        form = Form(request.POST)
        if form.is_valid():
            save(request.cadence_app, form.cleaned_data)
            return redirect('rhythms-index')
        else:
            # TODO(rescrv): Alert on this particular pass.
            pass
    else:
        form = Form()
    context = { 'email': request.user.email, 'form': form }
    return render(request, templ, context)

@login_required
def add_daily(request):
    def save(cadence_app, cleaned_data):
        cadence_app.add_daily(cleaned_data['desc'])
    return _add_rhythm(request, forms.DailyRhythmForm, 'rhythms/add_daily.html', save)

@login_required
def add_monthly(request):
    def save(cadence_app, cleaned_data):
        cadence_app.add_monthly(cleaned_data['desc'], cleaned_data['dotm'])
    return _add_rhythm(request, forms.MonthlyRhythmForm, 'rhythms/add_monthly.html', save)

@login_required
def add_week_daily(request):
    def save(cadence_app, cleaned_data):
        cadence_app.add_week_daily(cleaned_data['desc'], cleaned_data['dotw'])
    return _add_rhythm(request, forms.WeekDailyRhythmForm, 'rhythms/add_week_daily.html', save)

@login_required
def add_every_n_days(request):
    def save(cadence_app, cleaned_data):
        cadence_app.add_every_n_days(cleaned_data['desc'], cleaned_data['n'])
    return _add_rhythm(request, forms.EveryNDaysRhythmForm, 'rhythms/add_every_n.html', save)

@login_required
def edit(request, rhythm_id):
    email = request.user.email
    conn = sqlite3.connect('cadence.db')
    cadence_app = CadenceApp(conn, email)
    rhythm = [r for r in cadence_app.list_rhythms() if r.id == rhythm_id]
    if not rhythm:
        raise Http404
    rhythm = rhythm[0]
    next_page = request.GET.get('next', '')
    if request.method == "POST":
        form = form_for_rhythm(rhythm)(request.POST, initial=rhythm.as_dict())
        if form.is_valid():
            if not form.cleaned_data['rhythm_id']:
                form.cleaned_data['rhythm_id'] = rhythm.id
            if form.cleaned_data['rhythm_id'] != rhythm.id:
                # TODO(rescrv): Alert on this particular 404.
                raise Http404
            if form.has_changed():
                if isinstance(rhythm, cadence.Daily):
                    cadence_app.edit_daily(rhythm.id, desc=form.cleaned_data['desc'])
                elif isinstance(rhythm, cadence.Monthly):
                    cadence_app.edit_monthly(rhythm.id, desc=form.cleaned_data['desc'], dotm=form.cleaned_data['dotm'],
                            slider_before=form.cleaned_data['slider_before'],
                            slider_after=form.cleaned_data['slider_after'])
                elif isinstance(rhythm, cadence.WeekDaily):
                    cadence_app.edit_week_daily(rhythm.id, desc=form.cleaned_data['desc'], dotw=form.cleaned_data['dotw'],
                            slider_before=form.cleaned_data['slider_before'],
                            slider_after=form.cleaned_data['slider_after'])
                elif isinstance(rhythm, cadence.EveryNDays):
                    cadence_app.edit_every_n_days(rhythm.id, desc=form.cleaned_data['desc'], n=form.cleaned_data['n'],
                            slider_before=form.cleaned_data['slider_before'],
                            slider_after=form.cleaned_data['slider_after'])
                else:
                    # TODO(rescrv): Alert on this particular 404.
                    raise Http404
            if next_page == 'schedule':
                return redirect('full-schedule')
            if next_page == 'delinquent':
                return redirect('delinquent-rhythms')
            return redirect('rhythms-index')
        else:
            # TODO(rescrv): Alert on this particular pass.
            pass
    else:
        form = form_for_rhythm(rhythm)(initial=rhythm.as_dict())
    context = { 'email': email, 'rhythm': rhythm, 'form': form, 'next': next_page }
    return render(request, 'rhythms/edit.html', context)

@login_required
def delete(request, rhythm_id):
    email = request.user.email
    conn = sqlite3.connect('cadence.db')
    cadence_app = CadenceApp(conn, email)
    cadence_app.delete_rhythm(rhythm_id)
    return redirect('rhythms-index')
