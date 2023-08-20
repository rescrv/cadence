from django.urls import path

from . import views

urlpatterns = [
    path("", views.index, name='full-schedule'),
    path("delinquent/", views.delinquent, name='delinquent-rhythms'),
    path("done/", views.done, name='mark-done'),
    path("defer/", views.defer, name='mark-defer'),
    path("spoons/", views.spoons, name='spoons'),
]
