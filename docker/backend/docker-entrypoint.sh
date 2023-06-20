#!/bin/bash
cat <<EOF > /app/template.json
{
    "sql":"$DATABASE_URL",
    "redis": "$REDIS_URL",
    "listen": "$HOST:$PORT",
    "jwt_priv": "$JWT_SECRET",
    "mail_host": "$SMTP_HOST",
    "mail_port": $SMTP_PORT,
    "mail_username": "$SMTP_USER",
    "mail_password": "$SMTP_PASSWD",
    "from_email": "$SMTP_SENDER_MAIL",
    "from_name": "$SMPT_SENDER_NAME",
    "email_verification_expiry_time": $MAIL_VERIFICATION_TIMEOUT,
    "host": "$HOST_NAME",
    "secret_key": "$SECRET_KEY",
    "questions_per_page": $QUESTION_PER_PAGE,
    "users_per_page": $USER_PER_PAGE,
    "tags_per_page": $TAGS_PER_PAGE
}
EOF

/usr/bin/kunjika-backend
