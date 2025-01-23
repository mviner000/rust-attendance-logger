<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useToast } from '@/components/ui/toast/use-toast'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

const { toast } = useToast()
const username = ref('')
const password = ref('')
const email = ref('')
const fullName = ref('')

const emit = defineEmits(['user-created'])

async function handleSubmit() {
  try {
    const newUser = await invoke('create_user', {
      username: username.value,
      password: password.value,
      email: email.value || null,
      fullName: fullName.value || null
    }) as { username: string }

    toast({
      title: 'User Created',
      description: `User ${newUser.username} added successfully`
    })

    emit('user-created', newUser)

    // Reset form
    username.value = ''
    password.value = ''
    email.value = ''
    fullName.value = ''
  } catch (error) {
    toast({
      title: 'Error',
      description: String(error),
      variant: 'destructive'
    })
  }
}
</script>

<template>
  <form @submit.prevent="handleSubmit" class="space-y-4">
    <div>
      <Label>Username</Label>
      <Input v-model="username" required />
    </div>
    <div>
      <Label>Password</Label>
      <Input v-model="password" type="password" required />
    </div>
    <div>
      <Label>Email (Optional)</Label>
      <Input v-model="email" type="email" />
    </div>
    <div>
      <Label>Full Name (Optional)</Label>
      <Input v-model="fullName" />
    </div>
    <Button type="submit">Create User</Button>
  </form>
</template>