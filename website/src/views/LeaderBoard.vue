<script setup>
import PlayerScore from '@/components/PlayerScore.vue';
import { inject } from 'vue';

const sorted_players = inject('sorted_players');
</script>

<template>
  <main>
    <div class="px-2 md:px-4 mx-auto max-w-screen-lg">
      <h1 class="text-4xl my-6 font-black text-center">Leaderboard</h1>
      <template v-for="(player, idx) in sorted_players" :key="player.id">
        <player-score
          :rank="idx + 1"
          :player="player"
          :class="idx % 2 == 0 ? 'bg-base-content/10' : ''"
        ></player-score>
      </template>
    </div>
  </main>
</template>
