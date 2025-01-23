<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import UserCreationForm from './UserCreationForm.vue'
import { 
  Table, TableBody, TableCaption, 
  TableCell, TableHead, TableHeader, TableRow 
} from '@/components/ui/table'

interface User {
  id: number
  username: string
}

const users = ref<User[]>([])

async function fetchUsers() {
  users.value = await invoke('get_users')
}

function onUserCreated(newUser: User) {
  users.value.unshift(newUser)
}

onMounted(fetchUsers)
</script>

<template>
  <div class="grid grid-cols-2 gap-4">
    <UserCreationForm @user-created="onUserCreated" />
    
    <Table>
      <TableCaption>List of Registered Users</TableCaption>
      <TableHeader>
        <TableRow>
          <TableHead class="w-[100px]">ID</TableHead>
          <TableHead>Username</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        <TableRow v-for="user in users" :key="user.id">
          <TableCell class="font-medium">{{ user.id }}</TableCell>
          <TableCell>{{ user.username }}</TableCell>
        </TableRow>
      </TableBody>
    </Table>
  </div>
</template>