import sys
import psycopg
from config import settings    


def run_jobs() -> int:
    with psycopg.connect(settings.database_url) as conn:
        with conn.cursor() as cur:
            cur.execute("refresh materialized view weekly_tags_by_popularity")

        conn.commit()    
    return 0




if __name__ == "__main__":
        run_jobs()

