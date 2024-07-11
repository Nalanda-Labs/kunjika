--
-- PostgreSQL database dump
--

-- Dumped from database version 16.1
-- Dumped by pg_dump version 16.1

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: views_delete_old_rows(); Type: FUNCTION; Schema: public; Owner: shiv
--

CREATE FUNCTION public.views_delete_old_rows() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
  DELETE FROM views WHERE created_date < NOW() - INTERVAL '15 minute';
  RETURN NEW;
END;
$$;


ALTER FUNCTION public.views_delete_old_rows() OWNER TO shiv;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: post_tags; Type: TABLE; Schema: public; Owner: shiv
--

CREATE TABLE public.post_tags (
    id bigint NOT NULL,
    post_id bigint NOT NULL,
    tag_id bigint NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


ALTER TABLE public.post_tags OWNER TO shiv;

--
-- Name: tags; Type: TABLE; Schema: public; Owner: shiv
--

CREATE TABLE public.tags (
    id bigint NOT NULL,
    name character varying(32) NOT NULL,
    info character varying(1048576),
    post_count bigint DEFAULT 0,
    created_date timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_date timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


ALTER TABLE public.tags OWNER TO shiv;

--
-- Name: daily_tags_by_popularity; Type: MATERIALIZED VIEW; Schema: public; Owner: shiv
--

CREATE MATERIALIZED VIEW public.daily_tags_by_popularity AS
 SELECT post_tags.tag_id,
    count(post_tags.tag_id) AS count,
    t.name
   FROM (public.post_tags
     LEFT JOIN public.tags t ON ((t.id = post_tags.tag_id)))
  WHERE (post_tags.created_at > (CURRENT_DATE - '1 day'::interval))
  GROUP BY post_tags.tag_id, t.name
  ORDER BY (count(post_tags.tag_id))
  WITH NO DATA;


ALTER MATERIALIZED VIEW public.daily_tags_by_popularity OWNER TO shiv;

--
-- Name: post_tags_id_seq; Type: SEQUENCE; Schema: public; Owner: shiv
--

CREATE SEQUENCE public.post_tags_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.post_tags_id_seq OWNER TO shiv;

--
-- Name: post_tags_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: shiv
--

ALTER SEQUENCE public.post_tags_id_seq OWNED BY public.post_tags.id;


--
-- Name: posts; Type: TABLE; Schema: public; Owner: shiv
--

CREATE TABLE public.posts (
    id bigint NOT NULL,
    title character varying(256),
    description character varying(1000000) NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    visible boolean DEFAULT true NOT NULL,
    op_id bigint DEFAULT 0,
    votes bigint DEFAULT 0 NOT NULL,
    slug character varying(256),
    views bigint DEFAULT 0 NOT NULL,
    answer_accepted boolean DEFAULT false NOT NULL,
    answer_count bigint DEFAULT 0 NOT NULL,
    posted_by_id bigint NOT NULL,
    reply_to_id bigint,
    updated_by_id bigint
);


ALTER TABLE public.posts OWNER TO shiv;

--
-- Name: posts_id_seq; Type: SEQUENCE; Schema: public; Owner: shiv
--

CREATE SEQUENCE public.posts_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.posts_id_seq OWNER TO shiv;

--
-- Name: posts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: shiv
--

ALTER SEQUENCE public.posts_id_seq OWNED BY public.posts.id;


--
-- Name: tags_count; Type: MATERIALIZED VIEW; Schema: public; Owner: shiv
--

CREATE MATERIALIZED VIEW public.tags_count AS
 SELECT count(1) AS count
   FROM public.tags
  WITH NO DATA;


ALTER MATERIALIZED VIEW public.tags_count OWNER TO shiv;

--
-- Name: tags_id_seq; Type: SEQUENCE; Schema: public; Owner: shiv
--

CREATE SEQUENCE public.tags_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.tags_id_seq OWNER TO shiv;

--
-- Name: tags_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: shiv
--

ALTER SEQUENCE public.tags_id_seq OWNED BY public.tags.id;


--
-- Name: users; Type: TABLE; Schema: public; Owner: shiv
--

CREATE TABLE public.users (
    id bigint NOT NULL,
    username character varying(10) NOT NULL,
    email character varying(256) NOT NULL,
    password_hash character varying(256) NOT NULL,
    created_date timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    modified_date timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    status character varying(10) DEFAULT 'normal'::character varying NOT NULL,
    image_url character varying(512) NOT NULL,
    location character varying(128) DEFAULT ''::character varying NOT NULL,
    name character varying(128) DEFAULT ''::character varying NOT NULL,
    karma bigint DEFAULT 1 NOT NULL,
    title character varying(64),
    designation character varying(64),
    website character varying(256),
    git character varying(256),
    twitter character varying(256),
    email_verified boolean DEFAULT false NOT NULL,
    deleted boolean DEFAULT false NOT NULL,
    displayname character varying(64) DEFAULT ''::character varying NOT NULL
);


ALTER TABLE public.users OWNER TO shiv;

--
-- Name: users_id_seq; Type: SEQUENCE; Schema: public; Owner: shiv
--

CREATE SEQUENCE public.users_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.users_id_seq OWNER TO shiv;

--
-- Name: users_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: shiv
--

ALTER SEQUENCE public.users_id_seq OWNED BY public.users.id;


--
-- Name: views; Type: TABLE; Schema: public; Owner: shiv
--

CREATE TABLE public.views (
    userid bigint,
    ipaddress character varying(64),
    qid bigint,
    created_date timestamp with time zone DEFAULT now(),
    id bigint NOT NULL
) WITH (ttl_expire_after = '15 min', ttl_job_cron = '0,15,30,45 * * * *');


ALTER TABLE public.views OWNER TO shiv;

--
-- Name: views_id_seq; Type: SEQUENCE; Schema: public; Owner: shiv
--

CREATE SEQUENCE public.views_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.views_id_seq OWNER TO shiv;

--
-- Name: views_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: shiv
--

ALTER SEQUENCE public.views_id_seq OWNED BY public.views.id;


--
-- Name: weekly_tags_by_popularity; Type: MATERIALIZED VIEW; Schema: public; Owner: shiv
--

CREATE MATERIALIZED VIEW public.weekly_tags_by_popularity AS
 SELECT post_tags.tag_id,
    count(post_tags.tag_id) AS count,
    t.name
   FROM (public.post_tags
     LEFT JOIN public.tags t ON ((t.id = post_tags.tag_id)))
  WHERE (post_tags.created_at > (CURRENT_DATE - '7 days'::interval))
  GROUP BY post_tags.tag_id, t.name
  ORDER BY (count(post_tags.tag_id))
  WITH NO DATA;


ALTER MATERIALIZED VIEW public.weekly_tags_by_popularity OWNER TO shiv;

--
-- Name: post_tags id; Type: DEFAULT; Schema: public; Owner: shiv
--

ALTER TABLE ONLY public.post_tags ALTER COLUMN id SET DEFAULT nextval('public.post_tags_id_seq'::regclass);


--
-- Name: posts id; Type: DEFAULT; Schema: public; Owner: shiv
--

ALTER TABLE ONLY public.posts ALTER COLUMN id SET DEFAULT nextval('public.posts_id_seq'::regclass);


--
-- Name: tags id; Type: DEFAULT; Schema: public; Owner: shiv
--

ALTER TABLE ONLY public.tags ALTER COLUMN id SET DEFAULT nextval('public.tags_id_seq'::regclass);


--
-- Name: users id; Type: DEFAULT; Schema: public; Owner: shiv
--

ALTER TABLE ONLY public.users ALTER COLUMN id SET DEFAULT nextval('public.users_id_seq'::regclass);


--
-- Name: views id; Type: DEFAULT; Schema: public; Owner: shiv
--

ALTER TABLE ONLY public.views ALTER COLUMN id SET DEFAULT nextval('public.views_id_seq'::regclass);


--
-- Name: posts posts_pkey; Type: CONSTRAINT; Schema: public; Owner: shiv
--

ALTER TABLE ONLY public.posts
    ADD CONSTRAINT posts_pkey PRIMARY KEY (id);


--
-- Name: tags tags_name_key; Type: CONSTRAINT; Schema: public; Owner: shiv
--

ALTER TABLE ONLY public.tags
    ADD CONSTRAINT tags_name_key UNIQUE (name);


--
-- Name: tags tags_pkey; Type: CONSTRAINT; Schema: public; Owner: shiv
--

ALTER TABLE ONLY public.tags
    ADD CONSTRAINT tags_pkey PRIMARY KEY (id);


--
-- Name: users users_email_key; Type: CONSTRAINT; Schema: public; Owner: shiv
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_email_key UNIQUE (email);


--
-- Name: users users_name_key; Type: CONSTRAINT; Schema: public; Owner: shiv
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_name_key UNIQUE (username);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: shiv
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: post_tags_post_id_idx; Type: INDEX; Schema: public; Owner: shiv
--

CREATE INDEX post_tags_post_id_idx ON public.post_tags USING btree (post_id);


--
-- Name: post_tags_tag_id_idx; Type: INDEX; Schema: public; Owner: shiv
--

CREATE INDEX post_tags_tag_id_idx ON public.post_tags USING btree (tag_id);


--
-- Name: posts_op_id_idx; Type: INDEX; Schema: public; Owner: shiv
--

CREATE INDEX posts_op_id_idx ON public.posts USING btree (op_id);


--
-- Name: posts_updated_at_idx; Type: INDEX; Schema: public; Owner: shiv
--

CREATE INDEX posts_updated_at_idx ON public.posts USING btree (updated_at);


--
-- Name: tags_post_count_idx; Type: INDEX; Schema: public; Owner: shiv
--

CREATE INDEX tags_post_count_idx ON public.tags USING btree (post_count);


--
-- Name: users_karma_idx; Type: INDEX; Schema: public; Owner: shiv
--

CREATE INDEX users_karma_idx ON public.users USING btree (karma);


--
-- Name: views views_delete_old_rows_trigger; Type: TRIGGER; Schema: public; Owner: shiv
--

CREATE TRIGGER views_delete_old_rows_trigger AFTER INSERT ON public.views FOR EACH STATEMENT EXECUTE FUNCTION public.views_delete_old_rows();


--
-- Name: posts posts_posted_by_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: shiv
--

ALTER TABLE ONLY public.posts
    ADD CONSTRAINT posts_posted_by_id_fkey FOREIGN KEY (posted_by_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: posts posts_reply_to_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: shiv
--

ALTER TABLE ONLY public.posts
    ADD CONSTRAINT posts_reply_to_id_fkey FOREIGN KEY (reply_to_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: posts posts_updated_by_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: shiv
--

ALTER TABLE ONLY public.posts
    ADD CONSTRAINT posts_updated_by_id_fkey FOREIGN KEY (updated_by_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--

