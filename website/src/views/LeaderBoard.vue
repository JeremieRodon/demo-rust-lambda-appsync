<script setup>
import PlayerScore from '@/components/PlayerScore.vue';
import TeamScore from '@/components/TeamScore.vue';
import { inject } from 'vue';

const ranked_players = inject('ranked_players');
const ranked_teams = inject('ranked_teams');
</script>

<template>
  <main>
    <div class="px-2 md:px-4 mx-auto max-w-screen-lg">
      <h1 class="text-4xl my-6 font-black text-center">Leaderboard</h1>
      <h2 class="text-base-content/40 text-xl font-bold text-center">Teams</h2>
      <div class="max-w-4xl mx-auto">
        <template v-for="team in ranked_teams" :key="team.team_name">
          <team-score :team="team"></team-score>
        </template>
      </div>
      <h2 class="text-base-content/40 text-xl font-bold text-center mb-2">Players</h2>
      <template v-for="(player, idx) in ranked_players" :key="player.id">
        <player-score
          :player="player"
          :class="idx % 2 == 0 ? 'bg-base-content/10' : ''"
        ></player-score>
      </template>
    </div>
  </main>
</template>
