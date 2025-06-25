class Root {
    user = null;
    posts = null;
    metadata = null;

    constructor(self, user, posts, metadata) {
        self.user = user;
        self.posts = posts;
        self.metadata = metadata;
    }
}

class User {
    name = null;
    email = null;
    id = null;
    roles = null;
    is_active = null;
    profile = null;

    constructor(self, name, email, id, roles, is_active, profile) {
        self.name = name;
        self.email = email;
        self.id = id;
        self.roles = roles;
        self.is_active = is_active;
        self.profile = profile;
    }
}

class Profile {
    bio = null;
    social = null;
    preferences = null;

    constructor(self, bio, social, preferences) {
        self.bio = bio;
        self.social = social;
        self.preferences = preferences;
    }
}

class Social {
    github = null;
    linkedin = null;

    constructor(self, github, linkedin) {
        self.github = github;
        self.linkedin = linkedin;
    }
}

class Preferences {
    notifications = null;
    theme = null;

    constructor(self, notifications, theme) {
        self.notifications = notifications;
        self.theme = theme;
    }
}

class Notifications {
    push = null;
    email = null;
    sms = null;

    constructor(self, push, email, sms) {
        self.push = push;
        self.email = email;
        self.sms = sms;
    }
}

class Post {
    id = null;
    tags = null;
    title = null;
    comments = null;

    constructor(self, id, tags, title, comments) {
        self.id = id;
        self.tags = tags;
        self.title = title;
        self.comments = comments;
    }
}

class Comment {
    likes = null;
    message = null;
    user = null;

    constructor(self, likes, message, user) {
        self.likes = likes;
        self.message = message;
        self.user = user;
    }
}

class Metadata {
    generated_at = null;
    server = null;
    flags = null;

    constructor(self, generated_at, server, flags) {
        self.generated_at = generated_at;
        self.server = server;
        self.flags = flags;
    }
}
let root = new Root(
    new User(
        "Luis Ricardo Alves Santos",
        "ricardo@example.com",
        12345,
        ["admin", "editor"],
        true,
        new Profile(
            "Desenvolvedor full stack com experiência em Rust, Java e JS.",
            new Social(
                "https://github.com/luisricardo2825",
                "https://www.linkedin.com/in/luis-ricardo-alves-santos-6061b5216/"
            ),
            new Preferences(new Notifications(true, true, false), "dark")
        )
    ),
    [
        new Post(1, ["rust", "sistemas", "desempenho"], "Introdução ao Rust", [
            new Comment(12, "Ótimo post!", "joao123"),
            new Comment(8, "Muito bem explicado.", "ana_dev"),
        ]),
        new Post(2, ["java", "spring"], "Spring Boot na prática", []),
    ],
    new Metadata("2025-06-25T02:30:00Z", "api-v2", null)
);

Io.println(root);
