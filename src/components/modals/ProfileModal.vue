<template>
  <Modal v-if="modals.profile" @close="toggleModal('profile')">
    <div class="profile-modal">
      <h2>
        <font-awesome-icon icon="user-circle" />
        Your Profile
      </h2>

      <div v-if="isDiscordLinked" class="profile-content">
        <!-- Discord User Info -->
        <div class="profile-header">
          <div class="avatar-container">
            <img
              v-if="discordAvatar"
              :src="discordAvatar"
              alt="Discord Avatar"
              class="avatar"
            />
            <div v-else class="avatar-placeholder">
              <font-awesome-icon icon="user" />
            </div>
          </div>
          <div class="user-info">
            <h3 class="username">{{ discordUsername }}</h3>
            <p class="user-id">ID: {{ discordUserId }}</p>
          </div>
        </div>

        <!-- Stats Section -->
        <div class="stats-section">
          <h4>
            <font-awesome-icon icon="chart-bar" />
            Your Stats
          </h4>
          <div v-if="loading" class="loading">
            <font-awesome-icon icon="spinner" spin />
            Loading stats...
          </div>
          <div v-else-if="statsError" class="error">
            <font-awesome-icon icon="exclamation-triangle" />
            {{ statsError }}
          </div>
          <div v-else-if="stats" class="stats-grid">
            <div class="stat-card">
              <div class="stat-value">{{ stats.total_games || 0 }}</div>
              <div class="stat-label">Games Played</div>
            </div>
            <div class="stat-card">
              <div class="stat-value">{{ stats.wins || 0 }}</div>
              <div class="stat-label">Wins</div>
            </div>
            <div class="stat-card">
              <div class="stat-value">{{ winRate }}%</div>
              <div class="stat-label">Win Rate</div>
            </div>
            <div class="stat-card">
              <div class="stat-value">{{ stats.favorite_role || 'None' }}</div>
              <div class="stat-label">Favorite Role</div>
            </div>
          </div>
        </div>

        <!-- Session Info -->
        <div class="session-section">
          <h4>
            <font-awesome-icon icon="clock" />
            Session Info
          </h4>
          <div class="session-details">
            <div class="session-row">
              <span class="session-label">Logged in:</span>
              <span class="session-value">{{ sessionCreated }}</span>
            </div>
            <div class="session-row">
              <span class="session-label">Expires:</span>
              <span class="session-value">{{ sessionExpires }}</span>
            </div>
          </div>
        </div>

        <!-- Actions -->
        <div class="profile-actions">
          <button @click="handleLogout" class="logout-button">
            <font-awesome-icon icon="sign-out-alt" />
            Log Out
          </button>
        </div>
      </div>

      <div v-else class="profile-content not-logged-in">
        <div class="login-prompt">
          <font-awesome-icon icon="user-slash" size="3x" />
          <h3>Not Logged In</h3>
          <p>Log in with Discord to track your game stats and see your profile.</p>
          <button @click="handleLogin" class="login-button">
            <font-awesome-icon :icon="['fab', 'discord']" />
            Log in with Discord
          </button>
        </div>
      </div>
    </div>
  </Modal>
</template>

<script>
import Modal from "./Modal.vue";
import { mapGetters, mapState, mapMutations } from "vuex";

export default {
  components: {
    Modal,
  },
  data() {
    return {
      loading: false,
      stats: null,
      statsError: null,
      sessionCreatedAt: null,
      sessionExpiresAt: null,
    };
  },
  computed: {
    ...mapState(["modals"]),
    ...mapGetters("stats", ["isDiscordLinked"]),
    ...mapState("stats", [
      "discordUserId",
      "discordUsername",
      "statsToken",
      "statsSessionId",
      "baseUrl",
    ]),
    discordAvatar() {
      // Discord avatar URL format: https://cdn.discordapp.com/avatars/{user_id}/{avatar_hash}.png
      const avatarHash = localStorage.getItem("discordAvatar");
      if (avatarHash && this.discordUserId) {
        return `https://cdn.discordapp.com/avatars/${this.discordUserId}/${avatarHash}.png?size=128`;
      }
      return null;
    },
    winRate() {
      if (!this.stats || !this.stats.total_games) return 0;
      return Math.round((this.stats.wins / this.stats.total_games) * 100);
    },
    sessionCreated() {
      if (!this.sessionCreatedAt) return "Unknown";
      const date = new Date(this.sessionCreatedAt * 1000);
      return date.toLocaleDateString() + " " + date.toLocaleTimeString();
    },
    sessionExpires() {
      if (!this.sessionExpiresAt) return "Unknown";
      const date = new Date(this.sessionExpiresAt * 1000);
      const now = Date.now();
      const diff = date - now;
      
      if (diff < 0) return "Expired";
      
      const days = Math.floor(diff / (1000 * 60 * 60 * 24));
      if (days > 1) return `In ${days} days`;
      
      const hours = Math.floor(diff / (1000 * 60 * 60));
      if (hours > 1) return `In ${hours} hours`;
      
      return "Soon";
    },
  },
  mounted() {
    console.log("ProfileModal mounted, isDiscordLinked:", this.isDiscordLinked);
    if (this.isDiscordLinked) {
      this.fetchStats();
      this.fetchSessionInfo();
    }
  },
  methods: {
    ...mapMutations(["toggleModal"]),
    handleLogin() {
      // Close modal and trigger login
      this.toggleModal('profile');
      // Wait a tick then call the login method from Menu
      this.$nextTick(() => {
        const baseUrl = import.meta.env.PROD
          ? "https://api.hystericca.dev"
          : "http://localhost:8001";
        const redirectUri = encodeURIComponent(
          window.location.origin + "/auth/callback.html",
        );
        const authUrl = `${baseUrl}/auth/discord?redirect_uri=${redirectUri}`;
        window.location.href = authUrl;
      });
    },
    async fetchStats() {
      if (!this.discordUserId) return;

      this.loading = true;
      this.statsError = null;

      try {
        const response = await fetch(
          `${this.baseUrl}/v1/players/${this.discordUserId}/stats`,
          {
            headers: {
              "X-API-Key": this.statsToken || "",
            },
          }
        );

        if (!response.ok) {
          throw new Error("Failed to fetch stats");
        }

        this.stats = await response.json();
      } catch (error) {
        console.error("Error fetching stats:", error);
        this.statsError = "Unable to load stats";
      } finally {
        this.loading = false;
      }
    },
    async fetchSessionInfo() {
      if (!this.statsSessionId) return;

      try {
        const response = await fetch(
          `${this.baseUrl}/session/${this.statsSessionId}`,
          {
            headers: {
              Authorization: `Bearer ${this.statsToken}`,
            },
          }
        );

        if (response.ok) {
          const data = await response.json();
          this.sessionCreatedAt = data.created_at;
          this.sessionExpiresAt = data.expires_at;
        }
      } catch (error) {
        console.error("Error fetching session info:", error);
      }
    },
    async handleLogout() {
      try {
        const confirmed = await this.$root.$dialog.confirm(
          "Log out of Discord?\n\nThis will disable stat tracking and clear your session.",
          "Confirm Logout"
        );

        if (confirmed) {
          this.$store.dispatch("stats/logout");
          this.toggleModal("profile");
          // Reload to clear any cached state
          setTimeout(() => window.location.reload(), 500);
        }
      } catch (error) {
        console.error("Error in logout:", error);
        // Fallback: logout without confirmation
        this.$store.dispatch("stats/logout");
        this.toggleModal("profile");
        setTimeout(() => window.location.reload(), 500);
      }
    },
  },
};
</script>

<style scoped lang="scss">
.profile-modal {
  padding: 20px;
  max-width: 500px;
  color: white;

  h2 {
    font-size: 24px;
    margin: 0 0 20px 0;
    display: flex;
    align-items: center;
    gap: 10px;
  }
}

.profile-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

/* Profile Header */
.profile-header {
  display: flex;
  align-items: center;
  gap: 15px;
  padding: 15px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.avatar-container {
  flex-shrink: 0;

  .avatar {
    width: 80px;
    height: 80px;
    border-radius: 50%;
    border: 3px solid rgba(255, 255, 255, 0.2);
  }

  .avatar-placeholder {
    width: 80px;
    height: 80px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.1);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 36px;
    color: rgba(255, 255, 255, 0.5);
  }
}

.user-info {
  flex-grow: 1;

  .username {
    font-size: 20px;
    margin: 0 0 5px 0;
    font-weight: bold;
  }

  .user-id {
    font-size: 12px;
    color: rgba(255, 255, 255, 0.6);
    margin: 0;
    font-family: monospace;
  }
}

/* Stats Section */
.stats-section {
  h4 {
    font-size: 16px;
    margin: 0 0 10px 0;
    display: flex;
    align-items: center;
    gap: 8px;
    opacity: 0.8;
  }
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 10px;
}

.stat-card {
  background: rgba(255, 255, 255, 0.05);
  padding: 15px;
  border-radius: 8px;
  text-align: center;
  border: 1px solid rgba(255, 255, 255, 0.1);
  transition: transform 0.2s ease, background 0.2s ease;

  &:hover {
    transform: translateY(-2px);
    background: rgba(255, 255, 255, 0.08);
  }

  .stat-value {
    font-size: 24px;
    font-weight: bold;
    margin-bottom: 5px;
  }

  .stat-label {
    font-size: 12px;
    opacity: 0.7;
    text-transform: uppercase;
  }
}

/* Session Section */
.session-section {
  h4 {
    font-size: 16px;
    margin: 0 0 10px 0;
    display: flex;
    align-items: center;
    gap: 8px;
    opacity: 0.8;
  }
}

.session-details {
  background: rgba(255, 255, 255, 0.05);
  padding: 15px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.session-row {
  display: flex;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);

  &:last-child {
    border-bottom: none;
  }

  .session-label {
    font-size: 13px;
    opacity: 0.7;
  }

  .session-value {
    font-size: 13px;
    font-weight: 500;
    display: flex;
    align-items: center;
    gap: 5px;

    .enabled {
      color: #4caf50;
    }

    .disabled {
      color: #f44336;
    }
  }
}

/* Actions */
.profile-actions {
  display: flex;
  justify-content: center;
  padding-top: 10px;
}

.logout-button {
  background: rgba(244, 67, 54, 0.2);
  color: white;
  border: 1px solid rgba(244, 67, 54, 0.5);
  padding: 12px 24px;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: all 0.2s ease;

  &:hover {
    background: rgba(244, 67, 54, 0.3);
    border-color: rgba(244, 67, 54, 0.7);
    transform: translateY(-2px);
  }

  &:active {
    transform: translateY(0);
  }
}

/* Not Logged In State */
.not-logged-in {
  .login-prompt {
    text-align: center;
    padding: 40px 20px;

    svg {
      opacity: 0.3;
      margin-bottom: 20px;
    }

    h3 {
      font-size: 20px;
      margin: 0 0 10px 0;
    }

    p {
      color: rgba(255, 255, 255, 0.7);
      margin: 0 0 20px 0;
    }
  }

  .login-button {
    background: #5865f2;
    color: white;
    border: none;
    padding: 12px 24px;
    border-radius: 8px;
    font-size: 14px;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    transition: all 0.2s ease;

    &:hover {
      background: #4752c4;
      transform: translateY(-2px);
    }

    &:active {
      transform: translateY(0);
    }
  }
}

/* Loading & Error States */
.loading,
.error {
  text-align: center;
  padding: 20px;
  font-size: 14px;
  opacity: 0.7;
}

.error {
  color: #f44336;
}
</style>
