import datetime
from email.policy import default
from typing import Any, Optional, Type, Union

from tortoise import fields
from tortoise.models import Model


def to_naive(value: datetime.datetime) -> datetime.datetime:
    if value.tzinfo is None:
        return value

    value = value.astimezone(datetime.timezone.utc)

    return value.replace(tzinfo=None)


class NaiveDatetimeField(fields.DatetimeField):
    skip_to_python_if_native = True

    class _db_postgres:  # noqa
        SQL_TYPE = "TIMESTAMP"

    def to_python_value(self, value: Any) -> Optional[datetime.datetime]:
        value = super().to_python_value(value)

        if value is None:
            return value

        return to_naive(value)

    def to_db_value(
        self,
        value: Optional[datetime.datetime],
        instance: "Union[Type[Model], Model]",
    ) -> Optional[datetime.datetime]:
        value = super().to_db_value(value, instance)

        if value is None:
            return value

        return to_naive(value)


class Users(Model):

    id = fields.IntField(pk=True)
    email = fields.CharField(max_length=256, unique=True)
    username = fields.CharField(max_length=64, unique=True)
    password_hash = fields.CharField(max_length=128)
    salt = fields.CharField(max_length=128)
    email_verified = fields.BooleanField(default=False)
    active = fields.BooleanField(default=True)
    created_at = fields.DatetimeField(auto_now_add=True)
    updated_at = fields.DatetimeField(auto_now=True)
    name = fields.CharField(max_length=128, null=True)
    seen_notification_id = fields.IntField(default=0)
    last_posted_at = fields.DatetimeField(null=True)
    username_lower = fields.CharField(max_length=64, unique=True)
    last_seen_at = fields.DatetimeField(null=True)


class Tags(Model):

    id = fields.IntField(pk=True)
    name = fields.CharField(max_length=64, unique=True)
    post_count = fields.IntField(default=0)
    created_at = fields.DatetimeField(auto_now_add=True)
    updated_at = fields.DatetimeField(auto_now=True)
    info = fields.TextField(default="")
