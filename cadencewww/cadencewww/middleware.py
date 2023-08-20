import sqlite3

from django.utils.deprecation import MiddlewareMixin

from cadence import CadenceApp

class CadenceAppMiddleware(MiddlewareMixin):

    def process_request(self, request):
        if not hasattr(request, "session"):
            raise ImproperlyConfigured
        if hasattr(request, 'user') and hasattr(request.user, 'email'):
            email = request.user.email
            conn = sqlite3.connect('cadence.db')
            tz = request.COOKIES.get('cadence_user_tz', 'UTC')
            request.cadence_app = CadenceApp(conn, email, tz)
