###############################################################################
# Only SQL:2016, PostgreSQL, MariaDB/MySQL, SQLite
###############################################################################

# https://github.com/JakeWheat/sql-overview/blob/master/sql-2016-foundation-grammar.txt
#
# Copy <reserved word> into tmp.txt
# cat tmp.txt | grep -oe "\w*" > sql2016.txt

# Wikipedia
# Table: SQL:2016, DB2, Mimer SQL, MySQL, Oracle, PostgreSQL, MSSQL, Teradata
# All SQL reserved words

#echo 'Getting SQL:2016 reserved keywords...'
#
#curl -s https://en.wikipedia.org/wiki/SQL_reserved_words | pcregrep -M 'class="table-rh">.*\n</th>\n<td>SQL-2016</td>' | \
#  grep -oe 'class="table-rh">\w*' | awk -F ">" '{print $2}' | sort | uniq > sql2016.txt
#
#echo 'SQL:2016: DONE'

###############################################################################

# PostgreSQL

echo 'Getting PostgreSQL reserved keywords...'

curl -s https://www.postgresql.org/docs/13/sql-keywords-appendix.html | pcregrep -M '<td><code class="token">.*</code></td>.*\n.*<td>reserved' | \
  grep -oe '>\w*</code>' | awk -F ">|<" '{print $2}' | sort | uniq > postgresql.txt

echo 'PostgreSQL: DONE'

###############################################################################

# MariaDB

echo 'Getting MariaDB reserved keywords...'

curl -s https://mariadb.com/kb/en/reserved-words/ | grep -oe "<td>\w\w*</td>" | awk -F ">|<" '{print $3}' | sort | uniq > mariadb.txt

echo 'MariaDB: DONE'

# MySQL

echo 'Getting MySQL reserved keywords...'

curl -s https://dev.mysql.com/doc/refman/8.0/en/keywords.html | grep -oe "<code class=\"literal\">\w*</code> (R)" | awk -F ">|<" '{print $3}' | sort | uniq > mysql.txt

echo 'MariaDB: DONE'

###############################################################################

# SQLite

echo 'Getting SQLite keywords...'

curl -s https://www.sqlite.org/lang_keywords.html | grep -oe "<li>\w*</li>" | awk -F ">|<" '{print $3}' | sort | uniq > sqlite.txt

echo 'SQLite: DONE'

# =============================================================================

echo 'Merge and Deduplication'

cat sql2016.txt postgresql.txt mariadb.txt mysql.txt sqlite.txt | sort | uniq > total.txt

echo 'ALL: DONE'
