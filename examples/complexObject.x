class BaseEntity {
    toString(self) {
        return JSON.stringify(self, null, 2);
    }
}

class Root extends BaseEntity {
    user = null;
    posts = null;
    metadata = null;

    constructor(self, user, posts, metadata) {
        self.user = user;
        self.posts = posts;
        self.metadata = metadata;
    }

    totalLikes(self) {
        let sum = 0;
        for (let post of self.posts) {
            for (let comment of post.comments) {
                sum = sum + comment.likes;
            }
        }
        return sum;
    }

    postsWithComments(self) {
        let result = [];
        for (let post of self.posts) {
            if (post.comments.length > 0) {
                result.push(post);
            }
        }
        return result;
    }
}

class User extends BaseEntity {
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

    isAdmin(self) {
        for (let role of self.roles) {
            if (role == "admin") {
                return true;
            }
        }
        return false;
    }
}

class Profile extends BaseEntity {
    bio = null;
    social = null;
    preferences = null;

    constructor(self, bio, social, preferences) {
        self.bio = bio;
        self.social = social;
        self.preferences = preferences;
    }
}

class Social extends BaseEntity {
    github = null;
    linkedin = null;

    constructor(self, github, linkedin) {
        self.github = github;
        self.linkedin = linkedin;
    }

    allLinks(self) {
        let links = [];
        if (self.github != null) {
            links.push(self.github);
        }
        if (self.linkedin != null) {
            links.push(self.linkedin);
        }
        return links;
    }
}

class Preferences extends BaseEntity {
    notifications = null;
    theme = null;

    constructor(self, notifications, theme) {
        self.notifications = notifications;
        self.theme = theme;
    }
}

class Notifications extends BaseEntity {
    push = null;
    email = null;
    sms = null;

    constructor(self, push, email, sms) {
        self.push = push;
        self.email = email;
        self.sms = sms;
    }

    anyEnabled(self) {
        if (self.push) {
            return true;
        }
        if (self.email) {
            return true;
        }
        if (self.sms) {
            return true;
        }
        return false;
    }
}

class Post extends BaseEntity {
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

    commentCount(self) {
        return self.comments.length;
    }
}

class Comment extends BaseEntity {
    likes = null;
    message = null;
    user = null;

    constructor(self, likes, message, user) {
        self.likes = likes;
        self.message = message;
        self.user = user;
    }

    summary(self) {
        return self.user + " (" + self.likes + " likes): " + self.message;
    }
}

class Metadata extends BaseEntity {
    generated_at = null;
    server = null;
    flags = null;

    constructor(self, generated_at, server, flags) {
        self.generated_at = generated_at;
        self.server = server;
        self.flags = flags;
    }
}

// Instância
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
        new Post(2, ["java", "spring"], "Spring Boot na prática", [])
    ],
    new Metadata("2025-06-25T02:30:00Z", "api-v2", null)
);

// Demonstração

let isAdmin = root.user.isAdmin();
Io.println("Usuário é admin? " + isAdmin);
Io.println("Total de likes: " + root.totalLikes());
Io.println("Links sociais:");
for (let link of root.user.profile.social.allLinks()) {
    Io.println("- " + link);
}
Io.println("Comentários do primeiro post:");
for (let c of root.posts[0].comments) {
    Io.println("• " + c.summary());
}
Io.println("\nEstrutura:");
Io.println(root.toString());
