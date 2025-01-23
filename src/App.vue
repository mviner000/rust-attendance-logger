<!-- src/App.vue -->

<script setup lang="ts">
import DarkMode from "./components/DarkMode.vue";
import Greet from "./components/Greet.vue";
import UserTable from "./components/UserTable.vue";
</script>

<template>
  <DarkMode class="float-right" />
  <div class="container mx-auto p-4">
    <Greet class="mb-4" />
    <UserTable />
  </div>
</template>