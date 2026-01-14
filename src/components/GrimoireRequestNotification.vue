<template>
  <transition name="slide-fade">
    <div class="grimoire-request-notification" v-if="visible">
      <div class="notification-content">
        <font-awesome-icon icon="book" class="icon" />
        <div class="message">
          <strong>{{ requesterName }}</strong> is requesting grimoire access
        </div>
        <div class="actions">
          <button @click="decline" class="btn-decline" title="Decline">
            <font-awesome-icon icon="times" />
          </button>
          <button @click="approve" class="btn-approve" title="Approve">
            <font-awesome-icon icon="check" />
          </button>
        </div>
      </div>
    </div>
  </transition>
</template>

<script>
export default {
  name: 'GrimoireRequestNotification',
  props: {
    requesterName: {
      type: String,
      required: true,
    },
    requesterId: {
      type: String,
      required: true,
    },
  },
  data() {
    return {
      visible: true,
    };
  },
  methods: {
    approve() {
      // Gather actual grimoire data from Vuex state
      const players = this.$store.state.players.players;
      const grimoireData = {
        // Send full player data with roles
        players: players.map(player => ({
          name: player.name,
          id: player.id,
          role: player.role ? { id: player.role.id, team: player.role.team } : null,
          reminders: player.reminders || [],
          isDead: player.isDead,
          isVoteless: player.isVoteless,
          pronouns: player.pronouns,
        })),
        bluffs: this.$store.state.players.bluffs.map(bluff => bluff.id || null),
        npcs: this.$store.state.players.npcs.map(npc => ({ id: npc.id, name: npc.name })),
        edition: this.$store.state.edition.id,
      };
      
      this.$store.state.socket.sendGrimoireResponse(this.requesterId, true, grimoireData);
      
      // Send updated gamestate to all clients to ensure storyteller info persists
      this.$store.state.socket.sendGamestate();
      
      this.visible = false;
      this.$emit('close');
    },
    decline() {
      this.$store.state.socket.sendGrimoireResponse(this.requesterId, false);
      this.visible = false;
      this.$emit('close');
    },
  },
};
</script>

<style lang="scss" scoped>
.grimoire-request-notification {
  position: fixed;
  top: 80px;
  right: 20px;
  z-index: 1000;
  
  .notification-content {
    display: flex;
    align-items: center;
    gap: 12px;
    background: linear-gradient(
      135deg,
      rgba(0, 0, 0, 0.85) 0%,
      rgba(20, 20, 40, 0.85) 100%
    );
    backdrop-filter: blur(8px);
    border: 2px solid rgba(212, 175, 55, 0.4);
    border-radius: 12px;
    padding: 12px 16px;
    box-shadow:
      0 0 20px rgba(123, 44, 191, 0.4),
      0 4px 15px rgba(0, 0, 0, 0.8);
    min-width: 320px;
    
    .icon {
      font-size: 24px;
      color: rgba(212, 175, 55, 0.8);
    }
    
    .message {
      flex: 1;
      color: white;
      font-size: 14px;
      line-height: 1.4;
      
      strong {
        color: rgba(212, 175, 55, 1);
      }
    }
    
    .actions {
      display: flex;
      gap: 8px;
      
      button {
        width: 36px;
        height: 36px;
        border-radius: 8px;
        border: 2px solid;
        background: rgba(0, 0, 0, 0.3);
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 18px;
        transition: all 0.2s ease;
        
        &.btn-decline {
          border-color: rgba(255, 80, 80, 0.5);
          color: rgba(255, 80, 80, 0.8);
          
          &:hover {
            background: rgba(255, 80, 80, 0.2);
            border-color: rgba(255, 80, 80, 1);
            color: rgba(255, 80, 80, 1);
            transform: scale(1.1);
          }
        }
        
        &.btn-approve {
          border-color: rgba(80, 255, 120, 0.5);
          color: rgba(80, 255, 120, 0.8);
          
          &:hover {
            background: rgba(80, 255, 120, 0.2);
            border-color: rgba(80, 255, 120, 1);
            color: rgba(80, 255, 120, 1);
            transform: scale(1.1);
          }
        }
      }
    }
  }
}

.slide-fade-enter-active {
  transition: all 0.3s ease;
}

.slide-fade-leave-active {
  transition: all 0.3s ease;
}

.slide-fade-enter-from {
  transform: translateX(100%);
  opacity: 0;
}

.slide-fade-leave-to {
  transform: translateX(100%);
  opacity: 0;
}
</style>
