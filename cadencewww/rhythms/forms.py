from django import forms
from django.core.validators import MinValueValidator, MaxValueValidator, ValidationError
from django.forms import NumberInput

def rhythm_id_field():
    return forms.CharField(widget=forms.HiddenInput(), required=False)

def desc_field():
    return forms.CharField(
            label='Description',
            initial='A concise description of the rhythm.',
            required=False,
            widget=forms.TextInput(attrs={'class': 'pure-input-2-3'}))

def slider_before_field():
    return forms.IntegerField(
            label='Days Before',
            initial=0,
            validators=[MinValueValidator(0), MaxValueValidator(89)])

def slider_after_field():
    return forms.IntegerField(
            label='Days After',
            initial=0,
            validators=[MinValueValidator(0), MaxValueValidator(89)])

class DailyRhythmForm(forms.Form):
    rhythm_id = rhythm_id_field()
    desc = desc_field()

class MonthlyRhythmForm(forms.Form):
    rhythm_id = rhythm_id_field()
    desc = desc_field()
    dotm = forms.IntegerField(
            label='Day of the Month',
            initial=1,
            validators=[MinValueValidator(1), MaxValueValidator(31)])
    slider_before = slider_before_field()
    slider_after = slider_after_field()

    def clean(self):
        if self.cleaned_data['slider_before'] > 28:
            raise ValidationError('Cannot slide more than 28 days prior to a monthly rhythm.')
        if self.cleaned_data['slider_after'] > 28:
            raise ValidationError('Cannot slide more than 28 days after a monthly rhythm.')

class WeekDailyRhythmForm(forms.Form):
    rhythm_id = rhythm_id_field()
    desc = desc_field()
    dotw = forms.ChoiceField(
            label='Day of the Week',
            initial=0,
            choices=[(0, 'Monday'), (1, 'Tuesday'), (2, 'Wednesday'), (3, 'Thursday'), (4, 'Friday'), (5, 'Saturday'), (6, 'Sunday')])
    slider_before = slider_before_field()
    slider_after = slider_after_field()

    def clean(self):
        if self.cleaned_data['slider_before'] > 6:
            raise ValidationError('Cannot slide more than 6 days prior to a week-daily rhythm.')
        if self.cleaned_data['slider_after'] > 6:
            raise ValidationError('Cannot slide more than 6 days after a week-daily rhythm.')

class EveryNDaysRhythmForm(forms.Form):
    rhythm_id = rhythm_id_field()
    desc = desc_field()
    n = forms.IntegerField(
            label='Periodicity (days)',
            initial=2,
            validators=[MinValueValidator(2), MaxValueValidator(90)])
    slider_before = slider_before_field()
    slider_after = slider_after_field()

    def clean(self):
        n = self.cleaned_data.get('n', 2)
        if self.cleaned_data['slider_before'] > n - 1:
            raise ValidationError('Cannot slide more than {} days prior to an every-{}-days rhythm.'.format(n - 1, n))
        if self.cleaned_data['slider_after'] > n - 1:
            raise ValidationError('Cannot slide more than {} days after an every-{}-days rhythm.'.format(n - 1, n))
