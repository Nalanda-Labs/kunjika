import datetime
from email.policy import default
import json
from typing import Any, Optional, Type, Union, List

from tortoise import fields
from tortoise.fields.base import Field
from tortoise.models import Model


class IntArrayField(Field, list):
    """
    Int Array field specifically for PostgreSQL.

    This field can store list of int values.
    """

    SQL_TYPE = "int[]"

    def __init__(self, **kwargs):
        super().__init__(**kwargs)

    def to_db_value(
        self, value: List[int], instance: "Union[Type[Model], Model]"
    ) -> Optional[List[int]]:
        return value

    def to_python_value(self, value: Any) -> Optional[List[int]]:
        if isinstance(value, str):
            array = json.loads(value.replace("'", '"'))
            return [int(x) for x in array]
        return value


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

    id = fields.BigIntField(pk=True)
    email = fields.CharField(max_length=256, unique=True)
    username = fields.CharField(max_length=64, unique=True)
    password_hash = fields.CharField(max_length=128)
    salt = fields.CharField(max_length=128)
    email_verified = fields.BooleanField(default=False)
    active = fields.BooleanField(default=True)
    created_at = fields.DatetimeField(auto_now_add=True)
    updated_at = fields.DatetimeField(auto_now=True)
    name = fields.CharField(max_length=128, null=True)
    seen_notification_id = fields.BigIntField(default=0)
    last_posted_at = fields.DatetimeField(null=True)
    username_lower = fields.CharField(max_length=64, unique=True)
    last_seen_at = fields.DatetimeField(null=True)


class Tags(Model):

    id = fields.BigIntField(pk=True)
    name = fields.CharField(max_length=64, unique=True)
    post_count = fields.BigIntField(default=0)
    created_at = fields.DatetimeField(auto_now_add=True)
    updated_at = fields.DatetimeField(auto_now=True)
    info = fields.TextField(default="", null=True)


class Posts(Model):

    id = fields.BigIntField(pk=True)
    title = fields.CharField(max_length=256)
    description = fields.CharField(max_length=1000000)
    posted_by: fields.ForeignKeyRelation[Users] = fields.ForeignKeyField("models.Users", related_name="posted_by")
    created_at = fields.DatetimeField(auto_now_add=True)
    updated_at = fields.DatetimeField(auto_now=True, index=True)
    updated_by: fields.ForeignKeyRelation[Users] = fields.ForeignKeyField("models.Users", related_name="updated_by")
    visible1 = fields.BooleanField(default=True)
    op_id = fields.BigIntField()
    votes = fields.BigIntField(default=0)
    reply_to: fields.ForeignKeyRelation[Users] = fields.ForeignKeyField("models.Users", related_name="reply_to", null=True)
    slug = fields.CharField(max_length=256)
    views = fields.BigIntField(default=0)
    answer_accepted = fields.BooleanField(default=False)
    answer_count = fields.IntField(default=0)

class PostTags(Model):
    id = fields.BigIntField(pk=True)
    post_id = fields.BigIntField()
    tag_id = fields.BigIntField()
    created_at = fields.DatetimeField(auto_now_add=True)
    updated_at = fields.DatetimeField(auto_now=True, index=True)