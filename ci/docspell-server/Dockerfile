FROM docspell/restserver:nightly

COPY ./docspell-0.24.0-dump-h2-1.24.0-2021-07-13-2307.sql /opt/h2-dump.sql
RUN java -cp /opt/docspell-restserver/lib/com.h2database.h2* \
    -Dfile.encoding=UTF-8 org.h2.tools.RunScript \
    -url "jdbc:h2:///var/docspell/db;MODE=PostgreSQL;DATABASE_TO_LOWER=TRUE" \
    -user "sa" -password "" \
    -script /opt/h2-dump.sql
