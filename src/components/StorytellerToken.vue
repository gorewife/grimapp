<template>
  <!-- Only show if there's a storyteller -->
  <div class="storyteller-token" v-if="shouldShowStorytellerToken">
    <div class="storyteller-container">
      <div class="storyteller-avatar">
        <img 
          v-if="storytellerAvatar"
          :src="storytellerAvatar"
          :alt="storytellerName"
          class="avatar-image"
        />
        <div v-else class="avatar-placeholder">
          <font-awesome-icon icon="user" />
        </div>
      </div>
      
      <!-- EXACT copy of Player nameplate, might be able to optimize this -->
      <div
        class="name"
        @click="isMenuOpen = !isMenuOpen"
        :class="{ 
          active: isMenuOpen,
        }"
      >
        <div class="name-row">
          <span>{{ storytellerName }}</span>
        </div>
        <span class="pronouns" v-if="storytellerPronouns">{{ storytellerPronouns }}</span>
      </div>

      <!-- Menu for storyteller editing OR spectator requesting grimoire -->
      <transition name="fold">
        <ul class="menu" v-if="isMenuOpen">
          <!-- Storyteller menu options -->
          <template v-if="canEditStoryteller">
            <li @click="changePronouns">
              <font-awesome-icon icon="venus-mars" />
              Change Pronouns
            </li>
            <li @click="changeName">
              <font-awesome-icon icon="user-edit" />
              Rename
            </li>
          </template>
          
          <!-- Spectator request grimoire option (only for unseated players) -->
          <template v-else-if="session.isSpectator && session.claimedSeat === -1">
            <li @click="requestGrimoire" v-if="!grimoireRequestStatus">
              <font-awesome-icon icon="book" />
              Request Grim
            </li>
            <li v-else-if="grimoireRequestStatus === 'pending'" class="disabled">
              <font-awesome-icon icon="clock" />
              Awaiting {{ storytellerName }}'s response...
            </li>
            <li v-else-if="grimoireRequestStatus === 'declined'" class="disabled">
              <font-awesome-icon icon="times" />
              Request declined
            </li>
          </template>
        </ul>
      </transition>
    </div>
  </div>
</template>

<script>
import { mapState } from 'vuex';

export default {
  name: 'StorytellerToken',
  
  computed: {
    ...mapState(['session', 'stats']),
    
    // Determine if we should show the storyteller token
    shouldShowStorytellerToken() {
      // Show if there's an active session
      return !!this.session.sessionId;
    },
    
    // Can you edit the storyteller info? (only if you ARE the storyteller)
    canEditStoryteller() {
      return !this.session.isSpectator;
    },
    
    storytellerDiscordId() {
      // Always read from session state (unified for both ST and spectators)
      return this.session.storyteller.discordId || this.stats.discordUserId;
    },
    
    storytellerAvatar() {
      const { discordId, discordAvatar } = this.session.storyteller;
      if (discordId && discordAvatar) {
        return `https://cdn.discordapp.com/avatars/${discordId}/${discordAvatar}.png?size=128`;
      }
      // Fallback to own Discord avatar if you're the ST
      if (!this.session.isSpectator) {
        const avatarHash = localStorage.getItem("discordAvatar");
        if (avatarHash && this.stats.discordUserId) {
          return `https://cdn.discordapp.com/avatars/${this.stats.discordUserId}/${avatarHash}.png?size=128`;
        }
      }
      return null;
    },
    
    storytellerName() {
      return this.session.storyteller.name || this.stats.discordUsername || 'Storyteller';
    },
    
    storytellerPronouns() {
      return this.session.storyteller.pronouns || '';
    },
  },
  
  data() {
    return {
      isMenuOpen: false,
      grimoireRequestStatus: null, // null, 'pending', 'declined'
    };
  },
  
  watch: {
    // Watch for grimoire response from storyteller
    'session.grimoireResponse'(response) {
      if (response && this.session.isSpectator) {
        this.handleGrimoireResponse(response);
        this.$store.commit('session/clearGrimoireResponse');
      }
    }
  },
  
  methods: {
    async changePronouns() {
      if (!this.canEditStoryteller) return;
      
      const currentPronouns = this.session.storyteller.pronouns || '';
      const pronouns = await window.$dialog.prompt("Storyteller pronouns", currentPronouns);
      if (pronouns !== null) {
        localStorage.setItem('storytellerPronouns', pronouns);
        // Send update to all clients (also updates Vuex state)
        this.$store.state.socket.sendStorytellerPronouns(pronouns);
        this.isMenuOpen = false;
      }
    },
    
    async changeName() {
      const currentName = this.session.storyteller.name || this.stats.discordUsername || 'Storyteller';
      const name = await window.$dialog.prompt("Storyteller name", currentName);
      if (name !== null && name !== "") {
        localStorage.setItem('storytellerName', name);
        // Send update to all clients (also updates Vuex state)
        this.$store.state.socket.sendStorytellerName(name);
        this.isMenuOpen = false;
      }
    },
    
    requestGrimoire() {
      // Send request to storyteller
      const requesterName = this.stats.discordUsername || 'Guest';
      this.$store.state.socket.sendGrimoireRequest(requesterName);
      this.grimoireRequestStatus = 'pending';
      // Keep menu open so spectator can see "Awaiting response..." status
      // this.isMenuOpen = false;
      
      // Auto-clear declined status after 5 seconds
      setTimeout(() => {
        if (this.grimoireRequestStatus === 'declined') {
          this.grimoireRequestStatus = null;
        }
      }, 5000);
    },
    
    handleGrimoireResponse(response) {
      if (response.approved) {
        // Apply grimoire data from storyteller to Vuex state
        const { players, bluffs, npcs } = response.data;
        
        // Clear existing players and add new ones
        this.$store.commit('players/clear');
        players.forEach(playerData => {
          this.$store.commit('players/add', playerData.name);
          const player = this.$store.state.players.players[this.$store.state.players.players.length - 1];
          
          // Apply role if present
          if (playerData.role && playerData.role.id) {
            const role = this.$store.state.roles.get(playerData.role.id) || 
                        this.$store.getters.rolesJSONbyId.get(playerData.role.id);
            if (role) {
              this.$store.commit('players/update', { player, property: 'role', value: role });
            }
          }
          
          // Apply other properties
          if (playerData.reminders) {
            this.$store.commit('players/update', { player, property: 'reminders', value: playerData.reminders });
          }
          if (playerData.isDead) {
            this.$store.commit('players/update', { player, property: 'isDead', value: true });
          }
          if (playerData.isVoteless) {
            this.$store.commit('players/update', { player, property: 'isVoteless', value: true });
          }
          if (playerData.pronouns) {
            this.$store.commit('players/update', { player, property: 'pronouns', value: playerData.pronouns });
          }
        });
        
        // Set bluffs
        if (bluffs) {
          bluffs.forEach((bluffId, index) => {
            if (bluffId) {
              const role = this.$store.state.roles.get(bluffId) || 
                          this.$store.getters.rolesJSONbyId.get(bluffId);
              if (role) {
                this.$store.commit('players/setBluff', { index, role });
              }
            }
          });
        }
        
        // Set NPCs (fabled characters)
        if (npcs && npcs.length) {
          npcs.forEach(npc => {
            if (npc && npc.id) {
              const npcData = this.$store.state.npcs.get(npc.id);
              if (npcData) {
                this.$store.commit('players/setNpcs', { index: this.$store.state.players.npcs.length, npcs: npcData });
              }
            }
          });
        }
        
        this.grimoireRequestStatus = null;
        this.isMenuOpen = false;
      } else {
        this.grimoireRequestStatus = 'declined';
        // Auto-close menu and clear status after showing "declined" for 3 seconds
        setTimeout(() => {
          this.grimoireRequestStatus = null;
          this.isMenuOpen = false;
        }, 3000);
      }
    },
  },
};
</script>

<style lang="scss" scoped>
.storyteller-token {
  position: absolute;
  top: 20px;
  left: 20px;
  z-index: 100;
  
  .storyteller-container {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    
    .storyteller-avatar {
      width: 120px; /* Twice as big to cover nameplate width */
      height: 120px; /* Keep 1:1 square ratio */
      border-radius: 8px;
      overflow: hidden;
      border: 2px solid #444;
      display: flex;
      align-items: center;
      justify-content: center;
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.5);
      
      .avatar-image {
        width: 100%;
        height: 100%;
        object-fit: cover;
      }
      
      .avatar-placeholder {
        background: linear-gradient(135deg, #333, #555);
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        color: #999;
        font-size: 36px;
      }
    }
    
    /* EXACT COPY from Player.vue .name CSS */
    > .name {
      display: flex;
      flex-direction: column;
      justify-content: center;
      font-size: 120%;
      line-height: 120%;
      cursor: pointer;
      white-space: nowrap;
      background: linear-gradient(
        135deg,
        rgba(0, 0, 0, 0.25) 0%,
        rgba(0, 0, 0, 0.25) 100%
      );
      backdrop-filter: blur(4px);
      border: 2px solid rgba(212, 175, 55, 0.3);
      border-radius: 10px;
      box-shadow:
        0 0 15px rgba(123, 44, 191, 0.3),
        0 4px 10px rgba(0, 0, 0, 0.7);
      padding: 0 4px;
      text-align: center;
      min-width: 80px;
      
      &.read-only {
        cursor: default;
        opacity: 0.9;
      }

      svg {
        top: 3px;
        margin-right: 2px;
      }

      .name-row {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 4px;
      }

      .discord-indicator {
        width: 1.2em;
        height: 1.2em;
        color: #5865F2;
      }
      
      .pronouns {
        color: rgba(255, 255, 255, 0.6);
        font-size: 80%;
        margin-top: 2px;
      }
    }
    
    /* EXACT COPY from Player.vue .menu CSS */  
    > .menu {
      position: absolute;
      left: 50%;
      transform: translateX(-50%);
      top: 100%;
      margin-top: 5px;
      text-align: left;
      font-size: 90%;
      list-style-type: none;
      margin: 0;
      padding: 0;
      z-index: 200;
      background: linear-gradient(
        135deg,
        rgba(0, 0, 0, 0.25) 0%,
        rgba(0, 0, 0, 0.25) 100%
      );
      backdrop-filter: blur(4px);
      padding: 2px 5px;
      border-radius: 10px;
      border: 2px solid rgba(212, 175, 55, 0.3);
      cursor: pointer;
      box-shadow:
        0 0 20px rgba(123, 44, 191, 0.3),
        0 4px 10px rgba(0, 0, 0, 0.6);

      &:before {
        content: " ";
        width: 0;
        height: 0;
        position: absolute;
        border: 10px solid transparent;
        border-bottom-color: rgba(212, 175, 55, 0.3);
        top: -22px;
        left: 50%;
        transform: translateX(-50%);
      }

      li {
        padding: 4px 8px;
        color: white;
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 8px;
        
        &:hover:not(.disabled) {
          color: rgba(212, 175, 55, 1);
          text-shadow: 0 0 8px rgba(212, 175, 55, 0.4);
        }
        
        &.disabled {
          opacity: 0.6;
          cursor: default;
          pointer-events: none;
        }
      }
    }
  }
}

/* EXACT COPY of fold animation from Player.vue */
.fold-enter-active,
.fold-leave-active {
  transform-origin: center top;
  transition: transform 200ms ease-in-out, opacity 200ms ease-in-out;
}

.fold-enter,
.fold-leave-to {
  transform: perspective(200px) rotateX(-90deg);
  opacity: 0;
}

/* Responsive adjustments */
@media (max-width: 768px) {
  .storyteller-token {
    top: 10px;
    left: 10px;
    
    .storyteller-container {
      .storyteller-avatar {
        width: 80px; /* Scale down for mobile */
        height: 80px;
        
        .avatar-placeholder {
          font-size: 24px;
        }
      }
      
      > .name {
        font-size: 100%;
        padding: 0 3px;
        min-width: 60px;
        
        .name-row {
          gap: 3px;
        }
        
        .discord-indicator {
          width: 1em;
          height: 1em;
        }
        
        .pronouns {
          font-size: 70%;
        }
      }
      
      > .menu {
        font-size: 80%;
        
        li {
          padding: 3px 6px;
          gap: 6px;
        }
      }
    }
  }
}
</style>