from django.urls import path

from . import views

urlpatterns = [
    path("", views.index, name='rhythms-index'),
    path("add/", views.add, name='rhythms-add'),
    path("add/daily/", views.add_daily, name='rhythms-add-daily'),
    path("add/monthly/", views.add_monthly, name='rhythms-add-monthly'),
    path("add/week-daily/", views.add_week_daily, name='rhythms-add-week-daily'),
    path("add/every-n-days/", views.add_every_n_days, name='rhythms-add-every-n-days'),
    path("edit/<str:rhythm_id>/", views.edit, name='rhythms-edit'),
    path("delete/<str:rhythm_id>/", views.delete, name='rhythms-delete'),
]
