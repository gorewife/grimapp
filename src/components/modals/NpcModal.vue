<template>
  <Modal class="npc" v-if="isDisplayed" @close="toggleModal('npc')">
    <h3>Choose a {{ type }} character to add to the game:</h3>
    <ul class="tokens">
      <li v-for="role in displayedRoles" :key="role.id" @click="setNpcs(role)">
        <Token :role="role" />
      </li>
    </ul>
    <div class="button-group">
      <span
        class="button"
        :class="{ townsfolk: type === 'fabled' }"
        @click="type = 'fabled'"
      >
        Fabled
      </span>
      <span
        class="button"
        :class="{ townsfolk: type === 'loric' }"
        @click="type = 'loric'"
      >
        Loric
      </span>
    </div>
  </Modal>
</template>

<script>
import { mapMutations, mapState } from "vuex";
import Modal from "./Modal";
import Token from "../Token";

export default {
  components: { Token, Modal },
  computed: {
    ...mapState(["modals", "npcs", "grimoire"]),
    ...mapState("players", ["npcs"]),
    availableRoles() {
      const type = this.type;
      const characters = [];
      const playedNpcIds = new Set(this.$store.state.players.npcs.map(npc => npc.id));
      
      this.$store.state.npcs.forEach((role) => {
        // don't show npcs that are already in play
        if (
          (((!type || type === "fabled") && role.team === "fabled") ||
            ((!type || type === "loric") && role.team === "loric")) &&
          !playedNpcIds.has(role.id)
        ) {
          characters.push(role);
        }
      });
      return characters;
    },
    isDisplayed() {
      return this.modals.npc && this.availableRoles.length;
    },
    displayedRoles() {
      return this.availableRoles;
    },
  },
  data() {
    return {
      type: "fabled",
    };
  },
  methods: {
    setNpcs(role) {
      this.$store.commit("players/setNpcs", {
        npcs: role,
      });
      this.$store.commit("toggleModal", "npc");
    },
    ...mapMutations(["toggleModal"]),
  },
};
</script>

<style scoped lang="scss">
ul.tokens li {
  border-radius: 50%;
  width: 8vw;
  margin: 0.5%;
  transition: transform 200ms ease;
  will-change: transform;

  @media (orientation: portrait) {
    width: 8vh;
  }

  &:hover {
    transform: scale(1.15);
    z-index: 10;
  }
}
</style>
