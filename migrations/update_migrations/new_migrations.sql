CREATE TABLE IF NOT EXISTS public._sqlx_migrations (
    version bigint NOT NULL PRIMARY KEY,
    description text NOT NULL,
    installed_on timestamp with time zone DEFAULT now() NOT NULL,
    success boolean NOT NULL,
    checksum bytea NOT NULL,
    execution_time bigint NOT NULL
);

TRUNCATE TABLE public._sqlx_migrations;
INSERT INTO public._sqlx_migrations VALUES 
    (20250228100001, 'request head', '2025-06-25 09:42:57.458214+00', true, '\xe5640cf44dec8fbe6c0fa7cba0a2437f6801f7324631e81c1406872bbb95e18122698a48a5cc051e4b2e89f97939c0a3', 2979709),
    (20250228100002, 'request item', '2025-06-25 09:42:57.463653+00', true, '\xf82f50063ee9985771d5df44d0c44f2cff4268cb65fe4f5931f5b840eb5b3d5ec69f388c020ce068850f653ff2c94773', 2690798),
    (20250228100003, 'request partner', '2025-06-25 09:42:57.468778+00', true, '\x1d9e6f2b62666d40fb863e265caa51b3338eac74896c23e592cd7a268b964a1adbcddcf47ec8753318accdde42eade62', 2747447),
    (20250228100004, 'proposal head', '2025-06-25 09:42:57.474+00', true, '\xf7d1fec6c0f18214eb9164b7aafa988035cef2d720e5404ba372963879b52d98ddda4006ac15cb3297096014a00184c0', 3083519),
    (20250228100005, 'proposal item', '2025-06-25 09:42:57.479541+00', true, '\xc7d59fcb4d0e41c73edd35bb07385d785cfc59feee24873ef4bce84e8a59012bb08c5990b7de8a1d646f538b814f96a2', 2600337),
    (20250228100006, 'status history', '2025-06-25 09:42:57.48457+00', true, '\x5004f766ec02b22e089a63440f94b8dc98a31b47182d0ad4f41b2ed6ac5f0a57f8bbc7a32a7a178c69a119ca9acb4859', 2683407),
    (20250228100007, 'organization question', '2025-06-25 09:42:57.489367+00', true, '\x08fdd48a1fb7f88b34c97761a1e350c6ffe4e00004f9131521e7939eef16ee7a270bdcc71efd36402781479fe82fca8b', 1777255),
    (20250519100001, 'request subject purchased', '2025-06-25 09:42:57.492714+00', true, '\xc009294dc6b25760ead4e3008221fb4c64a5db800adac86f8fffac003c5b39f8055495f60a09ebd89cdf98d8633831c1', 1658455),
    (20250519100002, 'partner subject purchased', '2025-06-25 09:42:57.495886+00', true, '\xf275dc04d15dab5d0cffc7b938059064dfdd8be66790e5fcb63c92c421c1dbea3c20937b5f990428b1a3eea07af9396e', 1367464);
