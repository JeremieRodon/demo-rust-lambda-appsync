<script setup>
import PlayerScore from '@/components/PlayerScore.vue';
import { computed, inject } from 'vue';

const players = inject('players');

const sorted_players = computed(() => {
  const sorted_players = [...players.values()];
  sorted_players.sort((p1, p2) => p2.clicks - p1.clicks);
  return sorted_players;
});
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
