define_keyword! {
    /// Ansi SQL:2016 keywords.
    ///
    /// See [Online SQL:2016 grammar] for details.
    ///
    /// [Online SQL:2016 grammar]: https://jakewheat.github.io/sql-overview/sql-2016-foundation-grammar.html#key-word
    AnsiKeyword => {
        A,
        ABS,
        ABSOLUTE,
        ACOS,
        ACTION,
        ADA,
        ADD,
        ADMIN,
        AFTER,
        ALL,
        ALLOCATE,
        ALTER,
        ALWAYS,
        AND,
        ANY,
        ARE,
        ARRAY,
        ARRAY_AGG,
        ARRAY_MAX_CARDINALITY,
        AS,
        ASC,
        ASENSITIVE,
        ASIN,
        ASSERTION,
        ASSIGNMENT,
        ASYMMETRIC,
        AT,
        ATAN,
        ATOMIC,
        ATTRIBUTE,
        ATTRIBUTES,
        AUTHORIZATION,
        AVG,
        BEFORE,
        BEGIN,
        BEGIN_FRAME,
        BEGIN_PARTITION,
        BERNOULLI,
        BETWEEN,
        BIGINT,
        BINARY,
        BLOB,
        BOOLEAN,
        BOTH,
        BREADTH,
        BY,
        C,
        CALL,
        CALLED,
        CARDINALITY,
        CASCADE,
        CASCADED,
        CASE,
        CAST,
        CATALOG,
        CATALOG_NAME,
        CEIL,
        CEILING,
        CHAIN,
        CHAINING,
        CHAR,
        CHARACTER,
        CHARACTERISTICS,
        CHARACTER_LENGTH,
        CHARACTERS,
        CHARACTER_SET_CATALOG,
        CHARACTER_SET_NAME,
        CHARACTER_SET_SCHEMA,
        CHAR_LENGTH,
        CHECK,
        CLASSIFIER,
        CLASS_ORIGIN,
        CLOB,
        CLOSE,
        COALESCE,
        COBOL,
        COLLATE,
        COLLATION,
        COLLATION_CATALOG,
        COLLATION_NAME,
        COLLATION_SCHEMA,
        COLLECT,
        COLUMN,
        COLUMN_NAME,
        COLUMNS,
        COMMAND_FUNCTION,
        COMMAND_FUNCTION_CODE,
        COMMIT,
        COMMITTED,
        CONDITION,
        CONDITIONAL,
        CONDITION_NUMBER,
        CONNECT,
        CONNECTION,
        CONNECTION_NAME,
        CONSTRAINT,
        CONSTRAINT_CATALOG,
        CONSTRAINT_NAME,
        CONSTRAINTS,
        CONSTRAINT_SCHEMA,
        CONSTRUCTOR,
        CONTAINS,
        CONTINUE,
        CONVERT,
        COPY,
        CORR,
        CORRESPONDING,
        COS,
        COSH,
        COUNT,
        COVAR_POP,
        COVAR_SAMP,
        CREATE,
        CROSS,
        CUBE,
        CUME_DIST,
        CURRENT,
        CURRENT_CATALOG,
        CURRENT_DATE,
        CURRENT_DEFAULT_TRANSFORM_GROUP,
        CURRENT_PATH,
        CURRENT_ROLE,
        CURRENT_ROW,
        CURRENT_SCHEMA,
        CURRENT_TIME,
        CURRENT_TIMESTAMP,
        CURRENT_TRANSFORM_GROUP_FOR_TYPE,
        CURRENT_USER,
        CURSOR,
        CURSOR_NAME,
        CYCLE,
        DATA,
        DATE,
        DATETIME_INTERVAL_CODE,
        DATETIME_INTERVAL_PRECISION,
        DAY,
        DEALLOCATE,
        DEC,
        DECFLOAT,
        DECIMAL,
        DECLARE,
        DEFAULT,
        DEFAULTS,
        DEFERRABLE,
        DEFERRED,
        DEFINE,
        DEFINED,
        DEFINER,
        DEGREE,
        DELETE,
        DENSE_RANK,
        DEPTH,
        DEREF,
        DERIVED,
        DESC,
        DESCRIBE,
        DESCRIBE_CATALOG,
        DESCRIBE_NAME,
        DESCRIBE_PROCEDURE_SPECIFIC_CATALOG,
        DESCRIBE_PROCEDURE_SPECIFIC_NAME,
        DESCRIBE_PROCEDURE_SPECIFIC_SCHEMA,
        DESCRIBE_SCHEMA,
        DESCRIPTOR,
        DETERMINISTIC,
        DIAGNOSTICS,
        DISCONNECT,
        DISPATCH,
        DISTINCT,
        DOMAIN,
        DOUBLE,
        DROP,
        DYNAMIC,
        DYNAMIC_FUNCTION,
        DYNAMIC_FUNCTION_CODE,
        EACH,
        ELEMENT,
        ELSE,
        EMPTY,
        ENCODING,
        END,
        END_FRAME,
        END_PARTITION,
        ENFORCED,
        EQUALS,
        ERROR,
        ESCAPE,
        EVERY,
        EXCEPT,
        EXCLUDE,
        EXCLUDING,
        EXEC,
        EXECUTE,
        EXISTS,
        EXP,
        EXPRESSION,
        EXTERNAL,
        EXTRACT,
        FALSE,
        FETCH,
        FILTER,
        FINAL,
        FINISH,
        FINISH_CATALOG,
        FINISH_NAME,
        FINISH_PROCEDURE_SPECIFIC_CATALOG,
        FINISH_PROCEDURE_SPECIFIC_NAME,
        FINISH_PROCEDURE_SPECIFIC_SCHEMA,
        FINISH_SCHEMA,
        FIRST,
        FIRST_VALUE,
        FLAG,
        FLOAT,
        FLOOR,
        FOLLOWING,
        FOR,
        FOREIGN,
        FORMAT,
        FORTRAN,
        FOUND,
        FRAME_ROW,
        FREE,
        FROM,
        FULFILL,
        FULFILL_CATALOG,
        FULFILL_NAME,
        FULFILL_PROCEDURE_SPECIFIC_CATALOG,
        FULFILL_PROCEDURE_SPECIFIC_NAME,
        FULFILL_PROCEDURE_SPECIFIC_SCHEMA,
        FULFILL_SCHEMA,
        FULL,
        FUNCTION,
        FUSION,
        G,
        GENERAL,
        GENERATED,
        GET,
        GLOBAL,
        GO,
        GOTO,
        GRANT,
        GRANTED,
        GROUP,
        GROUPING,
        GROUPS,
        HAS_PASS_THROUGH_COLUMNS,
        HAS_PASS_THRU_COLS,
        HAVING,
        HIERARCHY,
        HOLD,
        HOUR,
        IDENTITY,
        IGNORE,
        IMMEDIATE,
        IMMEDIATELY,
        IMPLEMENTATION,
        IN,
        INCLUDING,
        INCREMENT,
        INDICATOR,
        INITIAL,
        INITIALLY,
        INNER,
        INOUT,
        INPUT,
        INSENSITIVE,
        INSERT,
        INSTANCE,
        INSTANTIABLE,
        INSTEAD,
        INT,
        INTEGER,
        INTERSECT,
        INTERSECTION,
        INTERVAL,
        INTO,
        INVOKER,
        IS,
        ISOLATION,
        IS_PRUNABLE,
        JOIN,
        JSON,
        JSON_ARRAY,
        JSON_ARRAYAGG,
        JSON_EXISTS,
        JSON_OBJECT,
        JSON_OBJECTAGG,
        JSON_QUERY,
        JSON_TABLE,
        JSON_TABLE_PRIMITIVE,
        JSON_VALUE,
        K,
        KEEP,
        KEY,
        KEY_MEMBER,
        KEYS,
        KEY_TYPE,
        LAG,
        LANGUAGE,
        LARGE,
        LAST,
        LAST_VALUE,
        LATERAL,
        LEAD,
        LEADING,
        LEFT,
        LENGTH,
        LEVEL,
        LIKE,
        LIKE_REGEX,
        LISTAGG,
        LN,
        LOCAL,
        LOCALTIME,
        LOCALTIMESTAMP,
        LOCATOR,
        LOG,
        LOG10,
        LOWER,
        M,
        MAP,
        MATCH,
        MATCHED,
        MATCHES,
        MATCH_NUMBER,
        MATCH_RECOGNIZE,
        MAX,
        MAXVALUE,
        MEMBER,
        MERGE,
        MESSAGE_LENGTH,
        MESSAGE_OCTET_LENGTH,
        MESSAGE_TEXT,
        METHOD,
        MIN,
        MINUTE,
        MINVALUE,
        MOD,
        MODIFIES,
        MODULE,
        MONTH,
        MORE,
        MULTISET,
        MUMPS,
        NAME,
        NAMES,
        NATIONAL,
        NATURAL,
        NCHAR,
        NCLOB,
        NESTED,
        NESTING,
        NEW,
        NEXT,
        NFC,
        NFD,
        NFKC,
        NFKD,
        NO,
        NONE,
        NORMALIZE,
        NORMALIZED,
        NOT,
        NTH_VALUE,
        NTILE,
        NULL,
        NULLABLE,
        NULLIF,
        NULLS,
        NUMBER,
        NUMERIC,
        OBJECT,
        OCCURRENCES_REGEX,
        OCTET_LENGTH,
        OCTETS,
        OF,
        OFFSET,
        OLD,
        OMIT,
        ON,
        ONE,
        ONLY,
        OPEN,
        OPTION,
        OPTIONS,
        OR,
        ORDER,
        ORDERING,
        ORDINALITY,
        OTHERS,
        OUT,
        OUTER,
        OUTPUT,
        OVER,
        OVERFLOW,
        OVERLAPS,
        OVERLAY,
        OVERRIDING,
        P,
        PAD,
        PARAMETER,
        PARAMETER_MODE,
        PARAMETER_NAME,
        PARAMETER_ORDINAL_POSITION,
        PARAMETER_SPECIFIC_CATALOG,
        PARAMETER_SPECIFIC_NAME,
        PARAMETER_SPECIFIC_SCHEMA,
        PARTIAL,
        PARTITION,
        PASCAL,
        PASS,
        PASSING,
        PAST,
        PATH,
        PATTERN,
        PER,
        PERCENT,
        PERCENTILE_CONT,
        PERCENTILE_DISC,
        PERCENT_RANK,
        PERIOD,
        PLACING,
        PLAN,
        PLI,
        PORTION,
        POSITION,
        POSITION_REGEX,
        POWER,
        PRECEDES,
        PRECEDING,
        PRECISION,
        PREPARE,
        PRESERVE,
        PRIMARY,
        PRIOR,
        PRIVATE,
        PRIVATE_PARAMETERS,
        PRIVATE_PARAMS_S,
        PRIVILEGES,
        PROCEDURE,
        PRUNE,
        PTF,
        PUBLIC,
        QUOTES,
        RANGE,
        RANK,
        READ,
        READS,
        REAL,
        RECURSIVE,
        REF,
        REFERENCES,
        REFERENCING,
        REGR_AVGX,
        REGR_AVGY,
        REGR_COUNT,
        REGR_INTERCEPT,
        REGR_R2,
        REGR_SLOPE,
        REGR_SXX,
        REGR_SXY,
        REGR_SYY,
        RELATIVE,
        RELEASE,
        REPEATABLE,
        RESPECT,
        RESTART,
        RESTRICT,
        RESULT,
        RET_ONLY_PASS_THRU,
        RETURN,
        RETURNED_CARDINALITY,
        RETURNED_LENGTH,
        RETURNED_OCTET_LENGTH,
        RETURNED_SQLSTATE,
        RETURNING,
        RETURNS,
        RETURNS_ONLY_PASS_THROUGH,
        REVOKE,
        RIGHT,
        ROLE,
        ROLLBACK,
        ROLLUP,
        ROUTINE,
        ROUTINE_CATALOG,
        ROUTINE_NAME,
        ROUTINE_SCHEMA,
        ROW,
        ROW_COUNT,
        ROW_NUMBER,
        ROWS,
        RUNNING,
        SAVEPOINT,
        SCALAR,
        SCALE,
        SCHEMA,
        SCHEMA_NAME,
        SCOPE,
        SCOPE_CATALOG,
        SCOPE_NAME,
        SCOPE_SCHEMA,
        SCROLL,
        SEARCH,
        SECOND,
        SECTION,
        SECURITY,
        SEEK,
        SELECT,
        SELF,
        SENSITIVE,
        SEQUENCE,
        SERIALIZABLE,
        SERVER_NAME,
        SESSION,
        SESSION_USER,
        SET,
        SETS,
        SHOW,
        SIMILAR,
        SIMPLE,
        SIN,
        SINH,
        SIZE,
        SKIP,
        SMALLINT,
        SOME,
        SOURCE,
        SPACE,
        SPECIFIC,
        SPECIFIC_NAME,
        SPECIFICTYPE,
        SQL,
        SQLEXCEPTION,
        SQLSTATE,
        SQLWARNING,
        SQRT,
        START,
        START_CATALOG,
        START_NAME,
        START_PROCEDURE_SPECIFIC_CATALOG,
        START_PROCEDURE_SPECIFIC_NAME,
        START_PROCEDURE_SPECIFIC_SCHEMA,
        START_SCHEMA,
        STATE,
        STATEMENT,
        STATIC,
        STDDEV_POP,
        STDDEV_SAMP,
        STRING,
        STRUCTURE,
        STYLE,
        SUBCLASS_ORIGIN,
        SUBMULTISET,
        SUBSET,
        SUBSTRING,
        SUBSTRING_REGEX,
        SUCCEEDS,
        SUM,
        SYMMETRIC,
        SYSTEM,
        SYSTEM_TIME,
        SYSTEM_USER,
        T,
        TABLE,
        TABLE_NAME,
        TABLESAMPLE,
        TABLE_SEMANTICS,
        TAN,
        TANH,
        TEMPORARY,
        THEN,
        THROUGH,
        TIES,
        TIME,
        TIMESTAMP,
        TIMEZONE_HOUR,
        TIMEZONE_MINUTE,
        TO,
        TOP_LEVEL_COUNT,
        TRAILING,
        TRANSACTION,
        TRANSACTION_ACTIVE,
        TRANSACTIONS_COMMITTED,
        TRANSACTIONS_ROLLED_BACK,
        TRANSFORM,
        TRANSFORMS,
        TRANSLATE,
        TRANSLATE_REGEX,
        TRANSLATION,
        TREAT,
        TRIGGER,
        TRIGGER_CATALOG,
        TRIGGER_NAME,
        TRIGGER_SCHEMA,
        TRIM,
        TRIM_ARRAY,
        TRUE,
        TRUNCATE,
        TYPE,
        UESCAPE,
        UNBOUNDED,
        UNCOMMITTED,
        UNCONDITIONAL,
        UNDER,
        UNION,
        UNIQUE,
        UNKNOWN,
        UNNAMED,
        UNNEST,
        UPDATE,
        UPPER,
        USAGE,
        USER,
        USER_DEFINED_TYPE_CATALOG,
        USER_DEFINED_TYPE_CODE,
        USER_DEFINED_TYPE_NAME,
        USER_DEFINED_TYPE_SCHEMA,
        USING,
        UTF16,
        UTF32,
        UTF8,
        VALUE,
        VALUE_OF,
        VALUES,
        VARBINARY,
        VARCHAR,
        VAR_POP,
        VAR_SAMP,
        VARYING,
        VERSIONING,
        VIEW,
        WHEN,
        WHENEVER,
        WHERE,
        WIDTH_BUCKET,
        WINDOW,
        WITH,
        WITHIN,
        WITHOUT,
        WORK,
        WRAPPER,
        WRITE,
        YEAR,
        ZONE
    }
}
