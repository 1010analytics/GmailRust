# Gmail API Rust Project

This Rust project interacts with the Gmail API to authenticate a user, fetch email messages, and process them to extract useful information such as message contents. It uses OAuth 2.0 for authentication and communicates with the Gmail API to list and retrieve emails.

## Prerequisites

To run this project, you will need:

- Rust installed on your system. [Install Rust](https://www.rust-lang.org/tools/install).
- Access to the [Google Cloud Console](https://console.cloud.google.com/).
- A valid Gmail account.
- A `credentials.json` file for OAuth 2.0 access.

## Getting Started

### Step 1: Setting up Google Cloud Console

1. Go to the [Google Cloud Console](https://console.cloud.google.com/).
2. Create a new project (or use an existing one).
3. Navigate to **API & Services** > **Library**.
4. Search for **Gmail API** and enable it for your project.
5. Go to **API & Services** > **Credentials**.
6. Click on **Create Credentials** > **OAuth 2.0 Client IDs**.
7. Configure the OAuth consent screen. You can set it to "Internal" if it is for personal use.
8. Once you create the OAuth 2.0 credentials, download the `credentials.json` file (rename the file to credentials.json if the JSON file has any other name) and place it in the root of your project directory.

### Step 2: Project Setup

1. Clone this repository or download the project files.
2. Place the `credentials.json` file in the root directory of the project.
3. Create a `.gitignore` file (if not already present) and add `credentials.json` to it to avoid accidentally committing your credentials to Git.

### Step 3: Install Dependencies

This project uses the following dependencies, which are already included in the `Cargo.toml` file:

- `tokio` for asynchronous runtime
- `warp` for handling HTTP requests (for OAuth callback)
- `yup-oauth2` for OAuth 2.0 authentication
- `hyper` for making HTTP requests
- `hyper-rustls` for secure HTTPS connections
- `serde` and `serde_json` for JSON parsing
- `scraper` for HTML parsing

You can install these dependencies by running:

```bash
cargo build
```

### Step 4: Running the project

1. Open a terminal and navigate to the project folder.
2. Run the following command:

```bash
cargo run
```

3. After running this command, you'll receive a URL in the terminal. Copy the `http://127.0.0.1:{port}` part of the URL, go to the Google Cloud Console, and in the **Authorized redirect URIs** section, add `http://127.0.0.1:{port}` with the correct port.
4. The project will start an OAuth 2.0 flow and open a web browser asking for Gmail permissions.
5. Once authenticated, it will automatically close the browser and start fetching email messages.
6. The email bodies (decoded) and all the content will be printed in the terminal.

#### Please let me know if you need any changes or if there's anything else you'd like me to do for this project.
