<template>
  <div class="exam-complete">
    <div class="card">
      <h1>Exam Complete</h1>
      <p>Thank you for completing the IELTS Speaking Mock Test.</p>
      <p>Your responses have been recorded.</p>
      <router-link to="/" class="home-link">Return to Home</router-link>
    </div>
  </div>
</template>

<script setup>
</script>

<style scoped>
.exam-complete {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #fda4af 0%, #fb923c 45%, #fbbf24 100%);
  padding: 24px;
}

.card {
  background: white;
  padding: 48px;
  border-radius: 24px;
  text-align: center;
  box-shadow: 0 30px 80px rgba(15, 23, 42, 0.2);
}

.card h1 {
  font-family: 'Fraunces', serif;
  font-size: 32px;
  margin-bottom: 12px;
  color: #0f172a;
}

.card p {
  color: #475569;
  margin-bottom: 8px;
}

.home-link {
  display: inline-flex;
  margin-top: 20px;
  padding: 12px 24px;
  border-radius: 999px;
  background: #0f172a;
  color: white;
  text-decoration: none;
  font-weight: 600;
}
</style>
